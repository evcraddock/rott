//! Identity and initialization management
//!
//! Handles first-run setup and device identity for ROTT.
//!
//! The root document ID serves as the user's identity. On first run,
//! users either create a new identity or join an existing one.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::Config;
use crate::document::RottDocument;
use crate::document_id::DocumentId;
use crate::storage::AutomergePersistence;

/// Identity manager for ROTT
///
/// Handles checking initialization state and setting up identity.
pub struct Identity {
    config: Config,
    persistence: AutomergePersistence,
}

/// Result of initialization
#[derive(Debug)]
pub struct InitResult {
    /// The root document ID
    pub root_id: DocumentId,
    /// Whether this was a new identity (vs joining existing)
    pub is_new: bool,
}

impl Identity {
    /// Create a new identity manager with default configuration
    pub fn new() -> Result<Self> {
        let config = Config::load().context("Failed to load configuration")?;
        Ok(Self::with_config(config))
    }

    /// Create a new identity manager with specific configuration
    pub fn with_config(config: Config) -> Self {
        let persistence = AutomergePersistence::new(config.clone());
        Self {
            config,
            persistence,
        }
    }

    /// Check if ROTT has been initialized (has a root document ID)
    ///
    /// Note: This returns true even if in "pending sync" state (has ID but no document).
    /// Use `is_pending_sync()` to check if sync is required.
    pub fn is_initialized(&self) -> bool {
        self.persistence.has_identity().unwrap_or(false)
    }

    /// Check if we have a local document (fully initialized, not pending sync)
    pub fn has_local_document(&self) -> bool {
        self.persistence.exists()
    }

    /// Check if we're in "pending sync" state
    ///
    /// This occurs after `rott init --join` when we have the ID but haven't
    /// synced the actual document from the server yet.
    pub fn is_pending_sync(&self) -> Result<bool> {
        self.persistence.is_pending_sync()
    }

    /// Get the root document ID if initialized
    pub fn root_id(&self) -> Result<Option<DocumentId>> {
        self.persistence.load_root_doc_id()
    }

    /// Get the config file path (for display purposes)
    pub fn config_path(&self) -> PathBuf {
        Config::config_file_path()
    }

    /// Get the data directory path (for display purposes)
    pub fn data_dir(&self) -> &PathBuf {
        &self.config.data_dir
    }

    /// Initialize with a new identity
    ///
    /// Creates a new root document with a random ID.
    /// Returns an error if already initialized.
    pub fn initialize_new(&self) -> Result<InitResult> {
        if self.is_initialized() {
            anyhow::bail!(
                "Already initialized. Use `rott device show` to see your root document ID."
            );
        }

        // Validate storage is accessible
        self.persistence
            .validate_storage()
            .context("Storage validation failed")?;

        // Create new document
        let mut doc = RottDocument::new();
        let root_id = *doc.id();

        // Save it
        self.persistence
            .save(&mut doc)
            .context("Failed to save root document")?;

        Ok(InitResult {
            root_id,
            is_new: true,
        })
    }

    /// Initialize by joining an existing identity
    ///
    /// Stores the provided root document ID for later sync.
    /// Does NOT create a local Automerge document - that will be pulled
    /// from the sync server on first sync.
    ///
    /// After calling this, the device is in "pending sync" state until
    /// sync completes successfully.
    ///
    /// Returns an error if already initialized.
    pub fn initialize_join(&self, root_id: DocumentId) -> Result<InitResult> {
        if self.is_initialized() {
            anyhow::bail!(
                "Already initialized. Use `rott device show` to see your root document ID."
            );
        }

        // Validate storage is accessible
        self.persistence
            .validate_storage()
            .context("Storage validation failed")?;

        // Only save the root document ID - do NOT create a local document
        // The actual document will be pulled from the sync server
        self.persistence
            .save_root_doc_id(&root_id)
            .context("Failed to save root document ID")?;

        Ok(InitResult {
            root_id,
            is_new: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(temp_dir: &TempDir) -> Config {
        Config {
            data_dir: temp_dir.path().to_path_buf(),
            sync_url: None,
            sync_enabled: false,
            favorite_tag: None,
            log_file: None,
        }
    }

    #[test]
    fn test_not_initialized_initially() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::with_config(test_config(&temp_dir));

        assert!(!identity.is_initialized());
        assert!(identity.root_id().unwrap().is_none());
    }

    #[test]
    fn test_initialize_new() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::with_config(test_config(&temp_dir));

        let result = identity.initialize_new().unwrap();
        assert!(result.is_new);

        // Should now be initialized
        assert!(identity.is_initialized());
        assert_eq!(identity.root_id().unwrap().unwrap(), result.root_id);
    }

    #[test]
    fn test_initialize_new_fails_if_already_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::with_config(test_config(&temp_dir));

        // First init should succeed
        identity.initialize_new().unwrap();

        // Second init should fail
        let result = identity.initialize_new();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Already initialized"));
    }

    #[test]
    fn test_initialize_join() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::with_config(test_config(&temp_dir));

        let join_id = DocumentId::new();
        let result = identity.initialize_join(join_id).unwrap();

        assert!(!result.is_new);
        assert_eq!(result.root_id, join_id);

        // Should now be initialized (has ID) but in pending sync state (no document)
        assert!(identity.is_initialized());
        assert!(!identity.has_local_document());
        assert!(identity.is_pending_sync().unwrap());
        assert_eq!(identity.root_id().unwrap().unwrap(), join_id);
    }

    #[test]
    fn test_initialize_join_fails_if_already_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let identity = Identity::with_config(test_config(&temp_dir));

        // First init
        identity.initialize_new().unwrap();

        // Join should fail
        let join_id = DocumentId::new();
        let result = identity.initialize_join(join_id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Already initialized"));
    }

    #[test]
    fn test_root_id_persists() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Initialize
        let identity1 = Identity::with_config(config.clone());
        let result = identity1.initialize_new().unwrap();
        let original_id = result.root_id;

        // Create new identity manager (simulates restart)
        let identity2 = Identity::with_config(config);
        assert!(identity2.is_initialized());
        assert_eq!(identity2.root_id().unwrap().unwrap(), original_id);
    }
}
