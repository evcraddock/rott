//! Sync client for automerge-repo-sync-server
//!
//! Provides WebSocket-based synchronization with a remote sync server.
//!
//! ## Protocol
//!
//! Uses the automerge-repo sync protocol:
//! 1. Connect via WebSocket
//! 2. Send peer ID and document ID
//! 3. Exchange Automerge sync messages
//! 4. Apply received changes
//!
//! ## Usage
//!
//! ### One-shot sync (CLI style)
//!
//! ```ignore
//! let client = SyncClient::new("ws://localhost:3030", doc_id);
//! client.sync_once(&mut doc).await?;
//! ```
//!
//! ### Persistent sync (TUI style)
//!
//! ```ignore
//! let handle = spawn_sync_task(config, doc, sync_state);
//! // Send commands
//! handle.command_tx.send(SyncCommand::PushChanges).await?;
//! // Receive events
//! while let Some(event) = handle.event_rx.recv().await {
//!     match event {
//!         SyncTaskEvent::DocumentUpdated => refresh_ui(),
//!         SyncTaskEvent::StatusChanged(status) => update_indicator(status),
//!         _ => {}
//!     }
//! }
//! ```

mod client;
mod message;
mod persistent;
mod state;

pub use client::{SyncClient, SyncEvent, SyncStatus};
pub use persistent::{
    spawn_sync_task, ConnectionStatus, PersistentSyncConfig, PersistentSyncHandle, SyncCommand,
    SyncTaskEvent,
};
pub use state::SyncState;
