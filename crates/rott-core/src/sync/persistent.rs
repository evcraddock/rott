//! Persistent sync connection
//!
//! Maintains a long-lived WebSocket connection for real-time sync.
//! Handles reconnection automatically with exponential backoff.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use automerge::sync::{Message as SyncMessage, SyncDoc};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use super::message::{ClientMessage, PeerId, ServerMessage};
use super::state::SyncState;
use crate::document::RottDocument;
use crate::document_id::DocumentId;

/// Commands sent to the sync task
#[derive(Debug, Clone)]
pub enum SyncCommand {
    /// Push local changes to server
    PushChanges,
    /// Shutdown the sync task
    Shutdown,
}

/// Events emitted by the sync task
#[derive(Debug, Clone)]
pub enum SyncTaskEvent {
    /// Connection status changed
    StatusChanged(ConnectionStatus),
    /// Document was updated from remote changes
    DocumentUpdated,
    /// Error occurred
    Error(String),
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected, not trying
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and idle
    Connected,
    /// Actively syncing
    Syncing,
}

/// Handle to control the persistent sync task
pub struct PersistentSyncHandle {
    /// Send commands to the sync task
    pub command_tx: mpsc::Sender<SyncCommand>,
    /// Receive events from the sync task
    pub event_rx: mpsc::Receiver<SyncTaskEvent>,
    /// Watch connection status
    pub status_rx: watch::Receiver<ConnectionStatus>,
}

/// Configuration for persistent sync
#[derive(Debug, Clone)]
pub struct PersistentSyncConfig {
    /// WebSocket URL
    pub url: String,
    /// Document ID to sync
    pub doc_id: DocumentId,
    /// Initial reconnect delay
    pub initial_reconnect_delay: Duration,
    /// Maximum reconnect delay
    pub max_reconnect_delay: Duration,
}

impl Default for PersistentSyncConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            doc_id: DocumentId::new(),
            initial_reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(30),
        }
    }
}

/// Spawn a persistent sync task
///
/// Returns a handle to control and monitor the sync task.
/// The task will automatically reconnect on disconnection.
pub fn spawn_sync_task(
    config: PersistentSyncConfig,
    doc: Arc<Mutex<RottDocument>>,
    sync_state: Arc<Mutex<SyncState>>,
) -> PersistentSyncHandle {
    let (command_tx, command_rx) = mpsc::channel(16);
    let (event_tx, event_rx) = mpsc::channel(64);
    let (status_tx, status_rx) = watch::channel(ConnectionStatus::Disconnected);

    tokio::spawn(sync_task_loop(
        config, doc, sync_state, command_rx, event_tx, status_tx,
    ));

    PersistentSyncHandle {
        command_tx,
        event_rx,
        status_rx,
    }
}

/// Main sync task loop with reconnection
async fn sync_task_loop(
    config: PersistentSyncConfig,
    doc: Arc<Mutex<RottDocument>>,
    sync_state: Arc<Mutex<SyncState>>,
    mut command_rx: mpsc::Receiver<SyncCommand>,
    event_tx: mpsc::Sender<SyncTaskEvent>,
    status_tx: watch::Sender<ConnectionStatus>,
) {
    let peer_id: PeerId = format!("rott-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let mut reconnect_delay = config.initial_reconnect_delay;

    loop {
        // Try to connect
        let _ = status_tx.send(ConnectionStatus::Connecting);
        let _ = event_tx
            .send(SyncTaskEvent::StatusChanged(ConnectionStatus::Connecting))
            .await;

        match connect_and_sync(
            &config,
            &peer_id,
            &doc,
            &sync_state,
            &mut command_rx,
            &event_tx,
            &status_tx,
        )
        .await
        {
            Ok(should_shutdown) => {
                if should_shutdown {
                    let _ = status_tx.send(ConnectionStatus::Disconnected);
                    let _ = event_tx
                        .send(SyncTaskEvent::StatusChanged(ConnectionStatus::Disconnected))
                        .await;
                    break;
                }
                // Connection closed normally, reset backoff
                reconnect_delay = config.initial_reconnect_delay;
            }
            Err(e) => {
                let _ = event_tx
                    .send(SyncTaskEvent::Error(format!("Connection error: {}", e)))
                    .await;
            }
        }

        // Update status to disconnected
        let _ = status_tx.send(ConnectionStatus::Disconnected);
        let _ = event_tx
            .send(SyncTaskEvent::StatusChanged(ConnectionStatus::Disconnected))
            .await;

        // Wait before reconnecting, but check for shutdown command
        tokio::select! {
            _ = tokio::time::sleep(reconnect_delay) => {
                // Exponential backoff
                reconnect_delay = (reconnect_delay * 2).min(config.max_reconnect_delay);
            }
            cmd = command_rx.recv() => {
                match cmd {
                    Some(SyncCommand::Shutdown) | None => break,
                    Some(SyncCommand::PushChanges) => {
                        // Will push after reconnecting
                    }
                }
            }
        }
    }
}

/// Connect and run sync loop until disconnection or shutdown
async fn connect_and_sync(
    config: &PersistentSyncConfig,
    peer_id: &str,
    doc: &Arc<Mutex<RottDocument>>,
    sync_state: &Arc<Mutex<SyncState>>,
    command_rx: &mut mpsc::Receiver<SyncCommand>,
    event_tx: &mpsc::Sender<SyncTaskEvent>,
    status_tx: &watch::Sender<ConnectionStatus>,
) -> Result<bool> {
    // Connect
    let (ws_stream, _) = connect_async(&config.url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send join message
    let join_msg = ClientMessage::join(peer_id);
    write.send(Message::Binary(join_msg.encode())).await?;

    // Wait for peer response
    let server_peer_id = wait_for_peer(&mut read).await?;

    // Connected successfully
    let _ = status_tx.send(ConnectionStatus::Connected);
    let _ = event_tx
        .send(SyncTaskEvent::StatusChanged(ConnectionStatus::Connected))
        .await;

    // Do initial sync
    let _ = status_tx.send(ConnectionStatus::Syncing);
    let _ = event_tx
        .send(SyncTaskEvent::StatusChanged(ConnectionStatus::Syncing))
        .await;

    do_sync(
        peer_id,
        &server_peer_id,
        &config.doc_id,
        doc,
        sync_state,
        &mut write,
        &mut read,
        event_tx,
    )
    .await?;

    let _ = status_tx.send(ConnectionStatus::Connected);
    let _ = event_tx
        .send(SyncTaskEvent::StatusChanged(ConnectionStatus::Connected))
        .await;

    // Main loop: wait for commands or incoming messages
    loop {
        tokio::select! {
            // Check for commands
            cmd = command_rx.recv() => {
                match cmd {
                    Some(SyncCommand::PushChanges) => {
                        let _ = status_tx.send(ConnectionStatus::Syncing);
                        let _ = event_tx.send(SyncTaskEvent::StatusChanged(ConnectionStatus::Syncing)).await;

                        do_sync(
                            peer_id,
                            &server_peer_id,
                            &config.doc_id,
                            doc,
                            sync_state,
                            &mut write,
                            &mut read,
                            event_tx,
                        ).await?;

                        let _ = status_tx.send(ConnectionStatus::Connected);
                        let _ = event_tx.send(SyncTaskEvent::StatusChanged(ConnectionStatus::Connected)).await;
                    }
                    Some(SyncCommand::Shutdown) => {
                        write.close().await.ok();
                        return Ok(true); // Signal shutdown
                    }
                    None => {
                        // Channel closed
                        write.close().await.ok();
                        return Ok(true);
                    }
                }
            }

            // Check for incoming messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Ok(ServerMessage::Sync { data, .. }) = ServerMessage::decode(&data) {
                            // Incoming sync from server
                            let _ = status_tx.send(ConnectionStatus::Syncing);
                            let _ = event_tx.send(SyncTaskEvent::StatusChanged(ConnectionStatus::Syncing)).await;

                            handle_incoming_sync(
                                peer_id,
                                &server_peer_id,
                                &config.doc_id,
                                &data,
                                doc,
                                sync_state,
                                &mut write,
                                event_tx,
                            ).await?;

                            let _ = status_tx.send(ConnectionStatus::Connected);
                            let _ = event_tx.send(SyncTaskEvent::StatusChanged(ConnectionStatus::Connected)).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // Connection closed
                        return Ok(false);
                    }
                    Some(Err(e)) => {
                        return Err(e.into());
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Wait for peer handshake response
async fn wait_for_peer(
    read: &mut futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<String> {
    let timeout = Duration::from_secs(10);
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            anyhow::bail!("Timeout waiting for sync server. Check that the server is running.");
        }

        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Ok(ServerMessage::Peer { sender_id, .. }) = ServerMessage::decode(&data) {
                            return Ok(sender_id);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        anyhow::bail!("Sync server closed connection during handshake");
                    }
                    Some(Err(e)) => {
                        anyhow::bail!("Sync connection error: {}", e);
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(remaining) => {
                anyhow::bail!("Timeout waiting for sync server. Check that the server is running.");
            }
        }
    }
}

/// Perform a sync exchange
#[allow(clippy::too_many_arguments)]
async fn do_sync<S>(
    peer_id: &str,
    server_peer_id: &str,
    doc_id: &DocumentId,
    doc: &Arc<Mutex<RottDocument>>,
    sync_state: &Arc<Mutex<SyncState>>,
    write: &mut futures_util::stream::SplitSink<S, Message>,
    read: &mut futures_util::stream::SplitStream<S>,
    event_tx: &mpsc::Sender<SyncTaskEvent>,
) -> Result<()>
where
    S: futures_util::Sink<Message> + futures_util::Stream + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + Send + Sync + 'static,
    <S as futures_util::Stream>::Item: Into<Result<Message, tokio_tungstenite::tungstenite::Error>>,
{
    // Generate and send initial sync message
    let initial_msg = {
        let mut doc_guard = doc.lock().await;
        let mut state_guard = sync_state.lock().await;
        let peer_state = state_guard.get_or_create(server_peer_id);
        let result = doc_guard
            .inner_mut()
            .sync()
            .generate_sync_message(peer_state)
            .map(|m| m.encode());
        result
    };

    if let Some(msg_bytes) = initial_msg {
        let request = ClientMessage::request(peer_id, server_peer_id, doc_id, msg_bytes);
        write.send(Message::Binary(request.encode())).await?;
    }

    // Process responses
    let sync_timeout = Duration::from_secs(10);
    let sync_deadline = tokio::time::Instant::now() + sync_timeout;

    loop {
        let remaining = sync_deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        tokio::select! {
            msg = read.next() => {
                let msg: Option<Result<Message, _>> = msg.map(|m| m.into());
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        match ServerMessage::decode(&data) {
                            Ok(ServerMessage::Sync { data, .. }) => {
                                let should_continue = process_sync_message(
                                    peer_id,
                                    server_peer_id,
                                    doc_id,
                                    &data,
                                    doc,
                                    sync_state,
                                    write,
                                    event_tx,
                                ).await?;

                                if !should_continue {
                                    break;
                                }
                            }
                            Ok(ServerMessage::DocUnavailable { .. }) => {
                                // Document doesn't exist on server, push ours
                                let msg_bytes = {
                                    let mut doc_guard = doc.lock().await;
                                    let mut state_guard = sync_state.lock().await;
                                    let peer_state = state_guard.get_or_create(server_peer_id);
                                    let result = doc_guard
                                        .inner_mut()
                                        .sync()
                                        .generate_sync_message(peer_state)
                                        .map(|m| m.encode());
                                    result
                                };

                                if let Some(bytes) = msg_bytes {
                                    let msg = ClientMessage::sync(peer_id, server_peer_id, doc_id, bytes);
                                    write.send(Message::Binary(msg.encode())).await?;
                                }
                            }
                            Ok(ServerMessage::Error { message, .. }) => {
                                let _ = event_tx.send(SyncTaskEvent::Error(message)).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        return Err(anyhow::anyhow!("WebSocket error: {}", e));
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(remaining) => {
                break;
            }
        }
    }

    // Save sync state
    let state_guard = sync_state.lock().await;
    state_guard.save().ok();

    Ok(())
}

/// Process an incoming sync message
#[allow(clippy::too_many_arguments)]
async fn process_sync_message<S>(
    peer_id: &str,
    server_peer_id: &str,
    doc_id: &DocumentId,
    data: &[u8],
    doc: &Arc<Mutex<RottDocument>>,
    sync_state: &Arc<Mutex<SyncState>>,
    write: &mut futures_util::stream::SplitSink<S, Message>,
    event_tx: &mpsc::Sender<SyncTaskEvent>,
) -> Result<bool>
where
    S: futures_util::Sink<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + Send + Sync + 'static,
{
    let Ok(sync_msg) = SyncMessage::decode(data) else {
        return Ok(true);
    };

    // Process message and generate response in one block
    let response_bytes = {
        let mut doc_guard = doc.lock().await;
        let mut state_guard = sync_state.lock().await;
        let peer_state = state_guard.get_or_create(server_peer_id);

        doc_guard
            .inner_mut()
            .sync()
            .receive_sync_message(peer_state, sync_msg)?;

        // Generate response if needed
        let result = doc_guard
            .inner_mut()
            .sync()
            .generate_sync_message(peer_state)
            .map(|m| m.encode());
        result
    };

    // Notify that document was updated
    let _ = event_tx.send(SyncTaskEvent::DocumentUpdated).await;

    // Send response if we have one
    if let Some(bytes) = response_bytes {
        let msg = ClientMessage::sync(peer_id, server_peer_id, doc_id, bytes);
        write.send(Message::Binary(msg.encode())).await?;
        Ok(true)
    } else {
        // No more messages, sync complete
        Ok(false)
    }
}

/// Handle incoming sync message (server-initiated)
#[allow(clippy::too_many_arguments)]
async fn handle_incoming_sync<S>(
    peer_id: &str,
    server_peer_id: &str,
    doc_id: &DocumentId,
    data: &[u8],
    doc: &Arc<Mutex<RottDocument>>,
    sync_state: &Arc<Mutex<SyncState>>,
    write: &mut futures_util::stream::SplitSink<S, Message>,
    event_tx: &mpsc::Sender<SyncTaskEvent>,
) -> Result<()>
where
    S: futures_util::Sink<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + Send + Sync + 'static,
{
    process_sync_message(
        peer_id,
        server_peer_id,
        doc_id,
        data,
        doc,
        sync_state,
        write,
        event_tx,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status() {
        assert_eq!(
            ConnectionStatus::Disconnected,
            ConnectionStatus::Disconnected
        );
        assert_ne!(ConnectionStatus::Connected, ConnectionStatus::Connecting);
    }

    #[test]
    fn test_sync_command() {
        let cmd = SyncCommand::PushChanges;
        match cmd {
            SyncCommand::PushChanges => {}
            SyncCommand::Shutdown => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_default_config() {
        let config = PersistentSyncConfig::default();
        assert_eq!(config.initial_reconnect_delay, Duration::from_secs(1));
        assert_eq!(config.max_reconnect_delay, Duration::from_secs(30));
    }
}
