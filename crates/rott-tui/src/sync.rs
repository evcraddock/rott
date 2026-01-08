//! Sync integration for TUI
//!
//! Provides sync operations and a background connection monitor.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rott_core::sync::{SyncClient, SyncState};
use rott_core::{Config, Store};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::app::SyncIndicator;

/// Commands sent to the sync task
#[derive(Debug)]
pub enum SyncCommand {
    /// Shutdown the sync task
    Shutdown,
}

/// Events from the sync task
#[derive(Debug)]
pub enum SyncEvent {
    /// Connection status changed
    Connected,
    /// Disconnected from server
    Disconnected,
    /// Remote changes detected - main thread should sync
    RemoteChanges,
    /// Error occurred
    Error(String),
}

/// Handle for controlling the background sync task
pub struct SyncHandle {
    pub command_tx: mpsc::Sender<SyncCommand>,
    pub event_rx: mpsc::Receiver<SyncEvent>,
}

/// Spawn a background task that periodically triggers sync
///
/// Since automerge-repo-sync-server doesn't push changes to passive listeners,
/// we poll for changes every few seconds instead.
pub fn spawn_sync_poller(poll_interval_secs: u64) -> SyncHandle {
    let (command_tx, command_rx) = mpsc::channel(16);
    let (event_tx, event_rx) = mpsc::channel(64);

    tokio::spawn(sync_poller_task(poll_interval_secs, command_rx, event_tx));

    SyncHandle {
        command_tx,
        event_rx,
    }
}

/// Background task that periodically signals to check for remote changes
async fn sync_poller_task(
    poll_interval_secs: u64,
    mut command_rx: mpsc::Receiver<SyncCommand>,
    event_tx: mpsc::Sender<SyncEvent>,
) {
    let interval = Duration::from_secs(poll_interval_secs);

    // Notify that we're "connected" (polling is active)
    let _ = event_tx.send(SyncEvent::Connected).await;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                // Time to poll - signal main thread to sync
                let _ = event_tx.send(SyncEvent::RemoteChanges).await;
            }
            cmd = command_rx.recv() => {
                if matches!(cmd, Some(SyncCommand::Shutdown) | None) {
                    break;
                }
            }
        }
    }
}

/// Spawn the background sync connection task (WebSocket listener)
/// Note: This doesn't work well with automerge-repo-sync-server as it
/// doesn't push to passive listeners. Use spawn_sync_poller instead.
#[allow(dead_code)]
pub fn spawn_sync_listener(url: String, doc_id: String) -> SyncHandle {
    let (command_tx, command_rx) = mpsc::channel(16);
    let (event_tx, event_rx) = mpsc::channel(64);

    tokio::spawn(sync_listener_task(url, doc_id, command_rx, event_tx));

    SyncHandle {
        command_tx,
        event_rx,
    }
}

/// Background task that maintains WebSocket connection
async fn sync_listener_task(
    url: String,
    doc_id: String,
    mut command_rx: mpsc::Receiver<SyncCommand>,
    event_tx: mpsc::Sender<SyncEvent>,
) {
    let mut reconnect_delay = Duration::from_secs(1);
    let max_reconnect_delay = Duration::from_secs(30);

    loop {
        // Try to connect
        match connect_and_listen(&url, &doc_id, &mut command_rx, &event_tx).await {
            Ok(should_shutdown) => {
                if should_shutdown {
                    break;
                }
                // Normal disconnect, reset backoff
                reconnect_delay = Duration::from_secs(1);
            }
            Err(_) => {
                // Connection error
            }
        }

        // Notify disconnection
        let _ = event_tx.send(SyncEvent::Disconnected).await;

        // Wait before reconnecting, checking for shutdown
        tokio::select! {
            _ = tokio::time::sleep(reconnect_delay) => {
                reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
            }
            cmd = command_rx.recv() => {
                if matches!(cmd, Some(SyncCommand::Shutdown) | None) {
                    break;
                }
            }
        }
    }
}

/// Connect and listen for messages until disconnect or shutdown
async fn connect_and_listen(
    url: &str,
    doc_id: &str,
    command_rx: &mut mpsc::Receiver<SyncCommand>,
    event_tx: &mpsc::Sender<SyncEvent>,
) -> Result<bool, ()> {
    // Connect
    let (ws_stream, _) = connect_async(url).await.map_err(|e| {
        let _ = event_tx.try_send(SyncEvent::Error(format!("Connect failed: {}", e)));
    })?;

    let (mut write, mut read) = ws_stream.split();

    // Send a minimal join message to establish connection
    let peer_id = format!("rott-listener-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let join_msg = create_join_message(&peer_id);
    write
        .send(Message::Binary(join_msg))
        .await
        .map_err(|_| ())?;

    // Wait for peer response to get server's peer ID
    let server_peer_id = loop {
        match tokio::time::timeout(Duration::from_secs(10), read.next()).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                if let Some(sender_id) = parse_peer_response(&data) {
                    break sender_id;
                }
            }
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => return Ok(false),
            Ok(Some(Err(_))) | Err(_) => return Err(()),
            _ => continue,
        }
    };

    // Send a request for the document to subscribe to changes
    let request_msg = create_request_message(&peer_id, &server_peer_id, doc_id);
    write
        .send(Message::Binary(request_msg))
        .await
        .map_err(|_| ())?;

    // Notify connected
    let _ = event_tx.send(SyncEvent::Connected).await;

    // Listen for messages
    loop {
        tokio::select! {
            // Check for shutdown command
            cmd = command_rx.recv() => {
                if matches!(cmd, Some(SyncCommand::Shutdown) | None) {
                    let _ = write.close().await;
                    return Ok(true);
                }
            }

            // Listen for incoming WebSocket messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(_data))) => {
                        // Any binary message indicates sync activity
                        let _ = event_tx.send(SyncEvent::RemoteChanges).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // Connection closed
                        return Ok(false);
                    }
                    Some(Err(_)) => {
                        // WebSocket error
                        return Err(());
                    }
                    _ => {
                        // Ping/pong/text - ignore
                    }
                }
            }
        }
    }
}

/// Create a CBOR join message (same format as rott-core)
fn create_join_message(peer_id: &str) -> Vec<u8> {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PeerMetadata {
        #[serde(skip_serializing_if = "Option::is_none")]
        storage_id: Option<String>,
        is_ephemeral: bool,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct JoinMessage {
        #[serde(rename = "type")]
        msg_type: String,
        sender_id: String,
        peer_metadata: PeerMetadata,
        supported_protocol_versions: Vec<String>,
    }

    let msg = JoinMessage {
        msg_type: "join".to_string(),
        sender_id: peer_id.to_string(),
        peer_metadata: PeerMetadata {
            storage_id: None,
            is_ephemeral: false,
        },
        supported_protocol_versions: vec!["1".to_string()],
    };

    let mut bytes = Vec::new();
    ciborium::into_writer(&msg, &mut bytes).expect("CBOR encoding failed");
    bytes
}

/// Create a request message to subscribe to a document
fn create_request_message(peer_id: &str, target_id: &str, doc_id: &str) -> Vec<u8> {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RequestMessage {
        #[serde(rename = "type")]
        msg_type: String,
        sender_id: String,
        target_id: String,
        document_id: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    let msg = RequestMessage {
        msg_type: "request".to_string(),
        sender_id: peer_id.to_string(),
        target_id: target_id.to_string(),
        document_id: doc_id.to_string(),
        data: vec![], // Empty initial sync message
    };

    let mut bytes = Vec::new();
    ciborium::into_writer(&msg, &mut bytes).expect("CBOR encoding failed");
    bytes
}

/// Parse a peer response to extract the server's peer ID
fn parse_peer_response(data: &[u8]) -> Option<String> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PeerResponse {
        #[serde(rename = "type")]
        msg_type: String,
        sender_id: String,
    }

    let response: PeerResponse = ciborium::from_reader(data).ok()?;
    if response.msg_type == "peer" {
        Some(response.sender_id)
    } else {
        None
    }
}

/// Perform a sync operation, returning the new sync indicator state
pub async fn do_sync(store: &mut Store) -> SyncIndicator {
    let config = store.config().clone();

    if !config.sync_enabled {
        return SyncIndicator::Disabled;
    }

    let Some(ref sync_url) = config.sync_url else {
        return SyncIndicator::Disabled;
    };

    // Create sync state with persistence
    let sync_state_path = config.data_dir.join("sync_state.json");
    let sync_state = SyncState::with_path(sync_state_path).unwrap_or_else(|_| SyncState::new());

    // Create sync client
    let client = SyncClient::new(sync_url, *store.root_id()).with_sync_state(sync_state);

    // Sync
    let doc = store.document_mut();
    match client.sync_once(doc).await {
        Ok(updated) => {
            if updated {
                // Rebuild projection after sync
                if store.rebuild_projection().is_err() {
                    return SyncIndicator::Error;
                }
            }
            SyncIndicator::Synced
        }
        Err(_) => SyncIndicator::Offline,
    }
}

/// Check if sync is enabled
pub fn is_sync_enabled(config: &Config) -> bool {
    config.sync_enabled && config.sync_url.is_some()
}
