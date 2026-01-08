//! Sync integration for TUI
//!
//! Uses rott-core's persistent sync for real-time bidirectional sync.

use std::sync::Arc;
use tokio::sync::Mutex;

use rott_core::sync::{
    spawn_sync_task, ConnectionStatus, PersistentSyncConfig, PersistentSyncHandle, SyncState,
};
use rott_core::{Config, Store};

use crate::app::SyncIndicator;

/// Spawn the persistent sync task
///
/// Returns a handle to control and monitor the sync task.
pub fn spawn_persistent_sync(store: &Store, config: &Config) -> Option<PersistentSyncHandle> {
    if !is_sync_enabled(config) {
        return None;
    }

    let sync_url = config.sync_url.as_ref()?;

    // Create sync state with persistence
    let sync_state_path = config.data_dir.join("sync_state.json");
    let sync_state = SyncState::with_path(sync_state_path).unwrap_or_else(|_| SyncState::new());

    // Get shared document from store
    let shared_doc = store.shared_document();

    // Create config for persistent sync
    let sync_config = PersistentSyncConfig {
        url: sync_url.clone(),
        doc_id: store.root_id(),
        ..Default::default()
    };

    // Spawn the sync task
    Some(spawn_sync_task(
        sync_config,
        shared_doc,
        Arc::new(Mutex::new(sync_state)),
    ))
}

/// Convert core ConnectionStatus to TUI SyncIndicator
pub fn status_to_indicator(status: ConnectionStatus) -> SyncIndicator {
    match status {
        ConnectionStatus::Disconnected => SyncIndicator::Offline,
        ConnectionStatus::Connecting => SyncIndicator::Syncing,
        ConnectionStatus::Connected => SyncIndicator::Synced,
        ConnectionStatus::Syncing => SyncIndicator::Syncing,
    }
}

/// Check if sync is enabled
pub fn is_sync_enabled(config: &Config) -> bool {
    config.sync_enabled && config.sync_url.is_some()
}
