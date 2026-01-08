//! Sync command handler

use anyhow::{bail, Result};

use rott_core::sync::{SyncClient, SyncState};
use rott_core::{Config, Store};

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

    let root_id = store.root_id();

    // Create sync client
    let client = SyncClient::new(sync_url, root_id).with_sync_state(sync_state);

    output.message(&format!("Syncing document {}...", root_id));

    // Get shared document and sync
    let shared_doc = store.shared_document();
    let mut doc = shared_doc.lock().await;
    match client.sync_once(&mut doc).await {
        Ok(updated) => {
            drop(doc); // Release lock before rebuilding projection
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

/// Sync quietly (for auto-sync) - no output on success
pub async fn sync_quiet(store: &mut Store, config: &Config) -> Result<()> {
    let Some(ref sync_url) = config.sync_url else {
        return Ok(());
    };

    // Create sync state with persistence
    let sync_state_path = config.data_dir.join("sync_state.json");
    let sync_state = SyncState::with_path(sync_state_path).unwrap_or_else(|_| SyncState::new());

    let root_id = store.root_id();

    // Create sync client
    let client = SyncClient::new(sync_url, root_id).with_sync_state(sync_state);

    // Get shared document and sync
    let shared_doc = store.shared_document();
    let mut doc = shared_doc.lock().await;
    let updated = client.sync_once(&mut doc).await?;
    drop(doc); // Release lock before rebuilding projection

    if updated {
        // Rebuild projection after sync
        store.rebuild_projection()?;
    }

    Ok(())
}
