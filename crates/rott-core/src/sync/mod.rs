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
//! ```ignore
//! let client = SyncClient::new("ws://localhost:3030", doc_id);
//! client.start().await?;
//! ```

mod client;
mod message;
mod state;

pub use client::{SyncClient, SyncEvent, SyncStatus};
pub use state::SyncState;
