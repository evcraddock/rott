//! Automerge document persistence
//!
//! Handles saving and loading Automerge documents to/from the filesystem.
//! Uses atomic writes (write to temp file, then rename) to prevent corruption.
//!
//! Storage location: `~/.local/share/rott/` (configurable via `Config`)
//!
//! Files:
//! - `document.automerge` - The Automerge binary document
//! - `root_doc_id` - The document ID (bs58check encoded)
//!
//! ## Error Handling
//!
//! - Disk full: Detected and reported with recovery suggestion
//! - Permission denied: Clear error message with path
//! - Corrupt documents: Backed up automatically, fresh document created
//! - Missing directories: Created automatically

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use super::error::{StorageError, StorageResult};
use crate::config::Config;
use crate::document::RottDocument;
use crate::document_id::DocumentId;

/// Persistence layer for Automerge documents
///
/// Provides atomic file operations for saving/loading documents.
pub struct AutomergePersistence {
    config: Config,
}

impl AutomergePersistence {
    /// Create a new persistence handler with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Load configuration from default location and create persistence handler
    pub fn with_default_config() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self::new(config))
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check if a document exists on disk
    pub fn exists(&self) -> bool {
        self.config.automerge_path().exists()
    }

    /// Save a document to disk using atomic write
    ///
    /// This writes to a temporary file first, then renames it to the target path.
    /// This ensures the file is never left in a partially-written state.
    pub fn save(&self, doc: &mut RottDocument) -> Result<()> {
        let bytes = doc.save();
        let target_path = self.config.automerge_path();

        atomic_write(&target_path, &bytes)
            .with_context(|| format!("Failed to save document to {:?}", target_path))?;

        // Also save the document ID for reference
        self.save_root_doc_id(doc.id())?;

        Ok(())
    }

    /// Load a document from disk
    ///
    /// Returns `None` if the document file doesn't exist.
    /// Returns an error if the file exists but can't be read or parsed.
    pub fn load(&self) -> Result<Option<RottDocument>> {
        let path = self.config.automerge_path();

        if !path.exists() {
            return Ok(None);
        }

        let bytes =
            fs::read(&path).with_context(|| format!("Failed to read document from {:?}", path))?;

        let doc = RottDocument::load(&bytes)
            .with_context(|| format!("Failed to parse document from {:?}", path))?;

        Ok(Some(doc))
    }

    /// Load an existing document or create a new one
    ///
    /// If a document exists on disk, it is loaded and returned.
    /// Otherwise, a new document is created, saved, and returned.
    pub fn load_or_create(&self) -> Result<RottDocument> {
        if let Some(doc) = self.load()? {
            return Ok(doc);
        }

        let mut doc = RottDocument::new();
        self.save(&mut doc)?;
        Ok(doc)
    }

    /// Save the root document ID to a separate file
    ///
    /// This provides a quick way to get the document ID without loading
    /// the entire Automerge document.
    pub fn save_root_doc_id(&self, id: &DocumentId) -> Result<()> {
        let path = self.config.root_doc_id_path();
        let content = id.to_bs58check();

        atomic_write(&path, content.as_bytes())
            .with_context(|| format!("Failed to save root doc ID to {:?}", path))?;

        Ok(())
    }

    /// Check if we're in "pending sync" state
    ///
    /// This occurs when the user has joined an existing identity but
    /// hasn't synced yet. We have the root document ID but no local
    /// Automerge document.
    pub fn is_pending_sync(&self) -> Result<bool> {
        let has_id = self.load_root_doc_id()?.is_some();
        let has_doc = self.exists();
        Ok(has_id && !has_doc)
    }

    /// Check if we have a root document ID (either from new or join)
    pub fn has_identity(&self) -> Result<bool> {
        Ok(self.load_root_doc_id()?.is_some())
    }

    /// Load the root document ID from disk
    ///
    /// Returns `None` if the ID file doesn't exist.
    pub fn load_root_doc_id(&self) -> Result<Option<DocumentId>> {
        let path = self.config.root_doc_id_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read root doc ID from {:?}", path))?;

        let id = DocumentId::from_bs58check(content.trim())
            .with_context(|| format!("Invalid document ID in {:?}", path))?;

        Ok(Some(id))
    }

    /// Get the Automerge URL for the stored document
    ///
    /// Returns `None` if no document has been saved yet.
    pub fn get_document_url(&self) -> Result<Option<String>> {
        self.load_root_doc_id()
            .map(|opt_id| opt_id.map(|id| id.to_url()))
    }

    /// Delete all stored data
    ///
    /// Removes the Automerge document and root doc ID.
    /// Use with caution!
    pub fn delete_all(&self) -> Result<()> {
        let paths = [self.config.automerge_path(), self.config.root_doc_id_path()];

        for path in paths {
            if path.exists() {
                fs::remove_file(&path).with_context(|| format!("Failed to delete {:?}", path))?;
            }
        }

        Ok(())
    }

    /// Load document with automatic recovery from corruption
    ///
    /// If the document is corrupted:
    /// 1. Creates a backup of the corrupted file
    /// 2. Creates a fresh document
    /// 3. Returns the fresh document with a warning
    ///
    /// Returns `(document, was_recovered)` where `was_recovered` is true
    /// if the document was corrupted and a fresh one was created.
    pub fn load_with_recovery(&self) -> Result<(RottDocument, bool)> {
        let path = self.config.automerge_path();

        if !path.exists() {
            return Ok((RottDocument::new(), false));
        }

        // Try to read the file
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                return Err(StorageError::from_io(e, path.clone()).into());
            }
        };

        // Try to parse the document
        match RottDocument::load(&bytes) {
            Ok(doc) => Ok((doc, false)),
            Err(_) => {
                // Document is corrupted - attempt recovery
                let backup_path = self.backup_corrupt_file(&path)?;

                // Log the corruption (in production, this would go to a proper logger)
                eprintln!(
                    "Warning: Document was corrupted. Backup saved to {:?}. Starting fresh.",
                    backup_path
                );

                // Create fresh document
                let doc = RottDocument::new();

                Ok((doc, true))
            }
        }
    }

    /// Load or create with automatic recovery
    ///
    /// Like `load_or_create` but handles corruption gracefully.
    pub fn load_or_create_with_recovery(&self) -> Result<(RottDocument, bool)> {
        let (doc, was_recovered) = self.load_with_recovery()?;

        // Save the document (whether new or recovered)
        if was_recovered || !self.exists() {
            let mut doc = doc;
            self.save(&mut doc)?;
            return Ok((doc, was_recovered));
        }

        Ok((doc, false))
    }

    /// Create a backup of a corrupted file
    fn backup_corrupt_file(&self, path: &Path) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}.corrupt.{}.backup",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("document"),
            timestamp
        );
        let backup_path = path.with_file_name(backup_name);

        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;

        Ok(backup_path)
    }

    /// Validate that the data directory is writable
    ///
    /// Creates the directory if it doesn't exist and tests write access.
    pub fn validate_storage(&self) -> StorageResult<()> {
        let data_dir = &self.config.data_dir;

        // Create directory if needed
        if !data_dir.exists() {
            fs::create_dir_all(data_dir).map_err(|e| StorageError::CreateDirectory {
                path: data_dir.clone(),
                source: e,
            })?;
        }

        // Test write access by creating a temp file
        let test_path = data_dir.join(".write_test");
        match File::create(&test_path) {
            Ok(_) => {
                // Clean up test file
                let _ = fs::remove_file(&test_path);
                Ok(())
            }
            Err(e) => Err(StorageError::from_io(e, data_dir.clone())),
        }
    }

    /// Get storage statistics
    pub fn storage_stats(&self) -> StorageStats {
        let doc_path = self.config.automerge_path();

        StorageStats {
            document_size: fs::metadata(&doc_path).map(|m| m.len()).ok(),
            document_exists: doc_path.exists(),
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Size of the Automerge document in bytes
    pub document_size: Option<u64>,
    /// Whether the document file exists
    pub document_exists: bool,
}

impl StorageStats {
    /// Total storage size in bytes
    pub fn total_size(&self) -> u64 {
        self.document_size.unwrap_or(0)
    }

    /// Format total size as human-readable string
    pub fn total_size_human(&self) -> String {
        let bytes = self.total_size();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Write data to a file atomically
///
/// 1. Write to a temporary file in the same directory
/// 2. Sync the file to disk
/// 3. Rename the temp file to the target path
///
/// This ensures the target file is never left in a partially-written state.
fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }

    // Create temp file in the same directory (for atomic rename)
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    let mut file = File::create(&temp_path)
        .with_context(|| format!("Failed to create temp file {:?}", temp_path))?;

    file.write_all(data)
        .with_context(|| format!("Failed to write to temp file {:?}", temp_path))?;

    // Sync to disk before rename
    file.sync_all()
        .with_context(|| format!("Failed to sync temp file {:?}", temp_path))?;

    // Atomic rename
    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", temp_path, path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Link;
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
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Initially no document
        assert!(!persistence.exists());
        assert!(persistence.load().unwrap().is_none());

        // Create and save a document
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://example.com");
        link.set_title("Example");
        doc.add_link(&link).unwrap();

        persistence.save(&mut doc).unwrap();
        assert!(persistence.exists());

        // Load and verify
        let loaded = persistence.load().unwrap().unwrap();
        let links = loaded.get_all_links().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com");
        assert_eq!(links[0].title, "Example");
    }

    #[test]
    fn test_load_or_create_new() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Should create new document
        let doc = persistence.load_or_create().unwrap();
        assert!(persistence.exists());

        // Document ID should be saved
        let loaded_id = persistence.load_root_doc_id().unwrap().unwrap();
        assert_eq!(*doc.id(), loaded_id);
    }

    #[test]
    fn test_load_or_create_existing() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Create initial document with data
        let mut doc = RottDocument::new();
        let original_id = *doc.id();
        let link = Link::new("https://rust-lang.org");
        doc.add_link(&link).unwrap();
        persistence.save(&mut doc).unwrap();

        // load_or_create should return existing document
        let loaded = persistence.load_or_create().unwrap();
        assert_eq!(*loaded.id(), original_id);
        assert_eq!(loaded.get_all_links().unwrap().len(), 1);
    }

    #[test]
    fn test_root_doc_id_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Initially no ID
        assert!(persistence.load_root_doc_id().unwrap().is_none());

        // Save document
        let mut doc = RottDocument::new();
        let doc_id = *doc.id();
        persistence.save(&mut doc).unwrap();

        // ID should be persisted
        let loaded_id = persistence.load_root_doc_id().unwrap().unwrap();
        assert_eq!(doc_id, loaded_id);
    }

    #[test]
    fn test_get_document_url() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Initially no URL
        assert!(persistence.get_document_url().unwrap().is_none());

        // Save document
        let mut doc = RottDocument::new();
        persistence.save(&mut doc).unwrap();

        // Should return automerge: URL
        let url = persistence.get_document_url().unwrap().unwrap();
        assert!(url.starts_with("automerge:"));
        assert_eq!(url, doc.url());
    }

    #[test]
    fn test_delete_all() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Create document
        let mut doc = RottDocument::new();
        persistence.save(&mut doc).unwrap();
        assert!(persistence.exists());

        // Delete all
        persistence.delete_all().unwrap();
        assert!(!persistence.exists());
        assert!(persistence.load_root_doc_id().unwrap().is_none());
    }

    #[test]
    fn test_atomic_write_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("file.txt");

        atomic_write(&nested_path, b"test data").unwrap();

        assert!(nested_path.exists());
        let content = fs::read_to_string(&nested_path).unwrap();
        assert_eq!(content, "test data");
    }

    #[test]
    fn test_multiple_saves_preserve_id() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Create and save
        let mut doc = persistence.load_or_create().unwrap();
        let original_id = *doc.id();

        // Add data and save again
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();
        persistence.save(&mut doc).unwrap();

        // Load and verify ID preserved
        let loaded = persistence.load().unwrap().unwrap();
        assert_eq!(*loaded.id(), original_id);
        assert_eq!(loaded.get_all_links().unwrap().len(), 1);
    }

    #[test]
    fn test_document_integrity_after_modifications() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Create document
        let mut doc = persistence.load_or_create().unwrap();

        // Add multiple items
        for i in 0..10 {
            let link = Link::new(format!("https://example{}.com", i));
            doc.add_link(&link).unwrap();
        }
        persistence.save(&mut doc).unwrap();

        // Reload and verify
        let loaded = persistence.load().unwrap().unwrap();
        assert_eq!(loaded.get_all_links().unwrap().len(), 10);

        // Modify and save again
        let mut loaded = loaded;
        let links = loaded.get_all_links().unwrap();
        loaded.delete_link(links[0].id).unwrap();
        persistence.save(&mut loaded).unwrap();

        // Final verification
        let final_doc = persistence.load().unwrap().unwrap();
        assert_eq!(final_doc.get_all_links().unwrap().len(), 9);
    }

    #[test]
    fn test_load_with_recovery_valid_document() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Create and save a valid document
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();
        persistence.save(&mut doc).unwrap();

        // Load with recovery - should succeed without recovery
        let (loaded, was_recovered) = persistence.load_with_recovery().unwrap();
        assert!(!was_recovered);
        assert_eq!(loaded.get_all_links().unwrap().len(), 1);
    }

    #[test]
    fn test_load_with_recovery_corrupt_document() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let persistence = AutomergePersistence::new(config.clone());

        // Write corrupt data to the document path
        let doc_path = config.automerge_path();
        fs::create_dir_all(doc_path.parent().unwrap()).unwrap();
        fs::write(&doc_path, b"this is not valid automerge data").unwrap();

        // Load with recovery - should recover
        let (doc, was_recovered) = persistence.load_with_recovery().unwrap();
        assert!(was_recovered);
        assert!(doc.get_all_links().unwrap().is_empty()); // Fresh document

        // Backup should exist
        let backups: Vec<_> = fs::read_dir(config.data_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".corrupt."))
            .collect();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_load_with_recovery_no_document() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // No document exists - should create new
        let (doc, was_recovered) = persistence.load_with_recovery().unwrap();
        assert!(!was_recovered);
        assert!(doc.get_all_links().unwrap().is_empty());
    }

    #[test]
    fn test_load_or_create_with_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let persistence = AutomergePersistence::new(config.clone());

        // Write corrupt data
        let doc_path = config.automerge_path();
        fs::create_dir_all(doc_path.parent().unwrap()).unwrap();
        fs::write(&doc_path, b"corrupt").unwrap();

        // Should recover and save new document
        let (doc, was_recovered) = persistence.load_or_create_with_recovery().unwrap();
        assert!(was_recovered);

        // New document should be saved
        assert!(persistence.exists());
        let loaded = persistence.load().unwrap().unwrap();
        assert_eq!(*loaded.id(), *doc.id());
    }

    #[test]
    fn test_validate_storage() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Should succeed - temp dir is writable
        persistence.validate_storage().unwrap();
    }

    #[test]
    fn test_validate_storage_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("a").join("b").join("c");
        let config = Config {
            data_dir: nested_dir.clone(),
            sync_url: None,
            sync_enabled: false,
            favorite_tag: None,
            log_file: None,
        };
        let persistence = AutomergePersistence::new(config);

        assert!(!nested_dir.exists());
        persistence.validate_storage().unwrap();
        assert!(nested_dir.exists());
    }

    #[test]
    fn test_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = AutomergePersistence::new(test_config(&temp_dir));

        // Initially nothing exists
        let stats = persistence.storage_stats();
        assert!(!stats.document_exists);
        assert_eq!(stats.total_size(), 0);

        // Create document
        let mut doc = RottDocument::new();
        persistence.save(&mut doc).unwrap();

        let stats = persistence.storage_stats();
        assert!(stats.document_exists);
        assert!(stats.document_size.unwrap() > 0);
    }

    #[test]
    fn test_storage_stats_human_readable() {
        let stats = StorageStats {
            document_size: Some(2048),
            document_exists: true,
        };
        assert_eq!(stats.total_size_human(), "2.0 KB");

        let stats = StorageStats {
            document_size: Some(1024 * 1024 + 512 * 1024),
            document_exists: true,
        };
        assert_eq!(stats.total_size_human(), "1.5 MB");
    }
}
