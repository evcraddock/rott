//! Sync client implementation
//!
//! WebSocket-based client for syncing with automerge-repo-sync-server.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use automerge::sync::{Message as SyncMessage, SyncDoc};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info, warn};

use super::message::{ClientMessage, PeerId, ServerMessage};
use super::state::SyncState;
use crate::document::RottDocument;
use crate::document_id::DocumentId;

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and ready
    Connected,
    /// Syncing in progress
    Syncing,
    /// Error state
    Error,
}

/// Events emitted by the sync client
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Connection status changed
    StatusChanged(SyncStatus),
    /// Document was updated from remote
    DocumentUpdated,
    /// Error occurred
    Error(String),
    /// Peer connected
    PeerConnected(String),
}

/// Sync client for automerge-repo-sync-server
pub struct SyncClient {
    /// Server URL
    url: String,
    /// Document ID to sync
    doc_id: DocumentId,
    /// Our peer ID
    peer_id: PeerId,
    /// Current connection status
    status: watch::Sender<SyncStatus>,
    /// Status receiver for external monitoring
    status_rx: watch::Receiver<SyncStatus>,
    /// Event channel
    event_tx: mpsc::UnboundedSender<SyncEvent>,
    /// Event receiver
    event_rx: Option<mpsc::UnboundedReceiver<SyncEvent>>,
    /// Sync state
    sync_state: Arc<Mutex<SyncState>>,
}

impl SyncClient {
    /// Create a new sync client
    pub fn new(url: &str, doc_id: DocumentId) -> Self {
        let (status_tx, status_rx) = watch::channel(SyncStatus::Disconnected);
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Generate a unique peer ID
        let peer_id = format!("rott-{}", &uuid::Uuid::new_v4().to_string()[..8]);

        Self {
            url: url.to_string(),
            doc_id,
            peer_id,
            status: status_tx,
            status_rx,
            event_tx,
            event_rx: Some(event_rx),
            sync_state: Arc::new(Mutex::new(SyncState::new())),
        }
    }

    /// Set sync state with persistence path
    pub fn with_sync_state(mut self, state: SyncState) -> Self {
        self.sync_state = Arc::new(Mutex::new(state));
        self
    }

    /// Get the current status
    pub fn status(&self) -> SyncStatus {
        *self.status_rx.borrow()
    }

    /// Subscribe to status changes
    pub fn subscribe_status(&self) -> watch::Receiver<SyncStatus> {
        self.status_rx.clone()
    }

    /// Take the event receiver (can only be called once)
    pub fn take_events(&mut self) -> Option<mpsc::UnboundedReceiver<SyncEvent>> {
        self.event_rx.take()
    }

    /// Get our peer ID
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Connect and sync once
    ///
    /// This is a one-shot sync - connects, syncs, then disconnects.
    pub async fn sync_once(&self, doc: &mut RottDocument) -> Result<bool> {
        info!("Starting sync to {}", self.url);
        self.set_status(SyncStatus::Connecting);

        // Connect
        let ws_stream = match self.connect().await {
            Ok(s) => s,
            Err(e) => {
                warn!("Sync connection failed: {}", e);
                self.set_status(SyncStatus::Error);
                self.emit(SyncEvent::Error(e.to_string()));
                return Err(e);
            }
        };

        self.set_status(SyncStatus::Connected);
        debug!("Connected to sync server");

        // Sync
        let result = self.do_sync(ws_stream, doc).await;

        self.set_status(SyncStatus::Disconnected);
        match &result {
            Ok(updated) => info!("Sync complete, document_updated={}", updated),
            Err(e) => warn!("Sync failed: {}", e),
        }

        result
    }

    /// Connect to the sync server
    async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        debug!("Connecting to {}", self.url);
        let (ws_stream, _response) = connect_async(&self.url)
            .await
            .context("Failed to connect to sync server")?;

        Ok(ws_stream)
    }

    /// Perform the sync protocol
    async fn do_sync(
        &self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        doc: &mut RottDocument,
    ) -> Result<bool> {
        let (mut write, mut read) = ws_stream.split();

        self.set_status(SyncStatus::Syncing);

        // Send join message
        let join_msg = ClientMessage::join(&self.peer_id);
        write.send(Message::Binary(join_msg.encode())).await?;

        // Wait for peer response and server peer ID
        let server_peer_id: String;
        let timeout = Duration::from_secs(10);
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                anyhow::bail!(
                    "Timeout waiting for sync server response ({}). Check server is running.",
                    self.url
                );
            }

            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Binary(data))) => {
                            match ServerMessage::decode(&data) {
                                Ok(ServerMessage::Peer { sender_id, .. }) => {
                                    server_peer_id = sender_id.clone();
                                    self.emit(SyncEvent::PeerConnected(sender_id));
                                    break;
                                }
                                Ok(ServerMessage::Error { message, .. }) => {
                                    anyhow::bail!("Server error: {}", message);
                                }
                                Ok(_) => {
                                    // Ignore other messages during handshake
                                }
                                Err(e) => {
                                    eprintln!("Failed to decode message: {:?}", e);
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            anyhow::bail!(
                                "Sync server ({}) closed connection during handshake",
                                self.url
                            );
                        }
                        Some(Err(e)) => {
                            anyhow::bail!("Sync connection error ({}): {}", self.url, e);
                        }
                        None => {
                            anyhow::bail!("Sync server ({}) closed connection", self.url);
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(remaining) => {
                    anyhow::bail!(
                        "Timeout waiting for sync server response ({}). Check server is running.",
                        self.url
                    );
                }
            }
        }

        // Generate initial sync message and send request
        let mut sync_state = self.sync_state.lock().await;
        let peer_sync_state = sync_state.get_or_create(&server_peer_id);

        if let Some(sync_msg) = doc
            .inner_mut()
            .sync()
            .generate_sync_message(peer_sync_state)
        {
            let request_msg = ClientMessage::request(
                &self.peer_id,
                &server_peer_id,
                &self.doc_id,
                sync_msg.encode(),
            );
            write.send(Message::Binary(request_msg.encode())).await?;
        }

        drop(sync_state);

        // Process sync responses
        let mut updated = false;
        let sync_timeout = Duration::from_secs(10);
        let sync_deadline = tokio::time::Instant::now() + sync_timeout;

        loop {
            let remaining = sync_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Binary(data))) => {
                            match ServerMessage::decode(&data) {
                                Ok(ServerMessage::Sync { sender_id, data, .. }) => {
                                    let (should_continue, was_updated) = self
                                        .handle_sync_message(&sender_id, data, doc, &mut write)
                                        .await?;
                                    if was_updated {
                                        updated = true;
                                    }
                                    if !should_continue {
                                        break;
                                    }
                                }
                                Ok(ServerMessage::DocUnavailable { .. }) => {
                                    // Document doesn't exist on server yet, upload it
                                    let mut sync_state = self.sync_state.lock().await;
                                    let peer_sync_state = sync_state.get_or_create(&server_peer_id);

                                    if let Some(sync_msg) = doc.inner_mut().sync().generate_sync_message(peer_sync_state) {
                                        let msg = ClientMessage::sync(
                                            &self.peer_id,
                                            &server_peer_id,
                                            &self.doc_id,
                                            sync_msg.encode(),
                                        );
                                        write.send(Message::Binary(msg.encode())).await?;
                                    }
                                }
                                Ok(ServerMessage::Error { message, .. }) => {
                                    self.emit(SyncEvent::Error(message));
                                    break;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to decode message: {:?}", e);
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => break,
                        Some(Err(e)) => {
                            return Err(anyhow::anyhow!("WebSocket error: {}", e));
                        }
                        None => break,
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(remaining) => {
                    break;
                }
            }
        }

        // Save sync state
        let sync_state = self.sync_state.lock().await;
        sync_state.save().ok();

        // Close connection
        write.close().await.ok();

        if updated {
            self.emit(SyncEvent::DocumentUpdated);
        }

        Ok(updated)
    }

    /// Handle a sync message from the server
    async fn handle_sync_message<S>(
        &self,
        sender_id: &str,
        data: Vec<u8>,
        doc: &mut RottDocument,
        write: &mut futures_util::stream::SplitSink<S, Message>,
    ) -> Result<(bool, bool)>
    where
        S: futures_util::Sink<Message> + Unpin,
        <S as futures_util::Sink<Message>>::Error: std::error::Error + Send + Sync + 'static,
    {
        // Decode the sync message
        let Ok(sync_msg) = SyncMessage::decode(&data) else {
            return Ok((true, false));
        };

        // Apply to our document
        let mut sync_state = self.sync_state.lock().await;
        let peer_state = sync_state.get_or_create(sender_id);

        doc.inner_mut()
            .sync()
            .receive_sync_message(peer_state, sync_msg)?;

        // Generate response
        if let Some(response) = doc.inner_mut().sync().generate_sync_message(peer_state) {
            let client_msg =
                ClientMessage::sync(&self.peer_id, sender_id, &self.doc_id, response.encode());
            write.send(Message::Binary(client_msg.encode())).await?;
            Ok((true, true))
        } else {
            // No more messages to send, sync complete
            Ok((false, true))
        }
    }

    fn set_status(&self, status: SyncStatus) {
        let _ = self.status.send(status);
        self.emit(SyncEvent::StatusChanged(status));
    }

    fn emit(&self, event: SyncEvent) {
        let _ = self.event_tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_client_new() {
        let doc_id = DocumentId::new();
        let client = SyncClient::new("ws://localhost:3030", doc_id);

        assert_eq!(client.status(), SyncStatus::Disconnected);
        assert!(client.peer_id().starts_with("rott-"));
    }

    #[test]
    fn test_sync_status() {
        let doc_id = DocumentId::new();
        let client = SyncClient::new("ws://localhost:3030", doc_id);

        let rx = client.subscribe_status();
        assert_eq!(*rx.borrow(), SyncStatus::Disconnected);
    }
}
