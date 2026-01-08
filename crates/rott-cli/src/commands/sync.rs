//! Sync command handler

use anyhow::{bail, Result};

use rott_core::sync::{SyncClient, SyncState};
use rott_core::Store;

use crate::output::Output;

/// Sync with the remote server
pub async fn sync(store: &mut Store, output: &Output) -> Result<()> {
    let config = store.config();

    if !config.sync_enabled {
        bail!(
            "Sync is not enabled. Enable it with:\n  \
             rott config set sync_enabled true\n  \
             rott config set sync_url ws://your-server:3030"
        );
    }

    let Some(ref sync_url) = config.sync_url else {
        bail!(
            "Sync URL not configured. Set it with:\n  \
             rott config set sync_url ws://your-server:3030"
        );
    };

    output.message("Connecting to sync server...");

    // Create sync state with persistence
    let sync_state_path = config.data_dir.join("sync_state.json");
    let sync_state = SyncState::with_path(sync_state_path).unwrap_or_else(|_| SyncState::new());

    // Create sync client
    let client = SyncClient::new(sync_url, *store.root_id()).with_sync_state(sync_state);

    output.message(&format!("Syncing document {}...", store.root_id()));

    // Get mutable access to document and sync
    let doc = store.document_mut();
    match client.sync_once(doc).await {
        Ok(updated) => {
            if updated {
                // Rebuild projection after sync
                store.rebuild_projection()?;
                output.success("Sync complete - document updated");

                // Show new counts
                let links = store.link_count()?;
                let notes = store.note_count()?;
                output.message(&format!("  Links: {}, Notes: {}", links, notes));
            } else {
                output.success("Sync complete - already up to date");
            }
        }
        Err(e) => {
            output.message(&format!("Sync failed: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
