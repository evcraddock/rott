//! Sync state persistence
//!
//! Stores sync state between sessions to enable efficient incremental sync.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use automerge::sync::State as AutomergeSyncState;

/// Persistent sync state
///
/// Stores Automerge sync states for each peer we've synced with.
/// This allows resuming sync efficiently without re-exchanging all data.
#[derive(Debug, Default)]
pub struct SyncState {
    /// Sync states for each peer
    peers: HashMap<String, AutomergeSyncState>,
    /// Path to persist state
    path: Option<PathBuf>,
}

impl SyncState {
    /// Create a new sync state (in-memory only)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a sync state that persists to disk
    pub fn with_path(path: PathBuf) -> Result<Self> {
        let mut state = Self {
            peers: HashMap::new(),
            path: Some(path.clone()),
        };

        // Load existing state if available
        if path.exists() {
            state.load()?;
        }

        Ok(state)
    }

    /// Get or create sync state for a peer
    pub fn get_or_create(&mut self, peer_id: &str) -> &mut AutomergeSyncState {
        self.peers
            .entry(peer_id.to_string())
            .or_default()
    }

    /// Get sync state for a peer (if exists)
    pub fn get(&self, peer_id: &str) -> Option<&AutomergeSyncState> {
        self.peers.get(peer_id)
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.path else {
            return Ok(());
        };

        // Serialize peer states
        let data: HashMap<String, Vec<u8>> = self
            .peers
            .iter()
            .map(|(k, v)| (k.clone(), v.encode()))
            .collect();

        let json = serde_json::to_string(&data)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, json).context("Failed to save sync state")?;
        Ok(())
    }

    /// Load state from disk
    fn load(&mut self) -> Result<()> {
        let Some(ref path) = self.path else {
            return Ok(());
        };

        let json = fs::read_to_string(path).context("Failed to read sync state")?;
        let data: HashMap<String, Vec<u8>> = serde_json::from_str(&json)?;

        for (peer_id, bytes) in data {
            if let Ok(state) = AutomergeSyncState::decode(&bytes) {
                self.peers.insert(peer_id, state);
            }
        }

        Ok(())
    }

    /// Clear all sync state
    pub fn clear(&mut self) {
        self.peers.clear();
    }

    /// Get number of peers we have state for
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_state_new() {
        let mut state = SyncState::new();
        let peer_state = state.get_or_create("peer-1");
        assert!(peer_state.their_heads.is_none());
    }

    #[test]
    fn test_sync_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("sync_state.json");

        // Create and save state
        {
            let mut state = SyncState::with_path(path.clone()).unwrap();
            let _peer = state.get_or_create("peer-1");
            state.save().unwrap();
        }

        // Load state
        {
            let state = SyncState::with_path(path).unwrap();
            assert_eq!(state.peer_count(), 1);
            assert!(state.get("peer-1").is_some());
        }
    }

    #[test]
    fn test_sync_state_clear() {
        let mut state = SyncState::new();
        state.get_or_create("peer-1");
        state.get_or_create("peer-2");
        assert_eq!(state.peer_count(), 2);

        state.clear();
        assert_eq!(state.peer_count(), 0);
    }
}
