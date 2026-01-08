//! Unified storage interface
//!
//! The `Store` manages the root document and coordinates between:
//! - Automerge (source of truth)
//! - SQLite (read-optimized queries)
//!
//! ## Root Document
//!
//! The root document is created on first run and contains all user data.
//! Its ID serves as the user's identity for sync purposes.
//!
//! ## Notes
//!
//! Notes are children of links, not standalone entities. To add a note,
//! first get the link, add the note to it, then update the link.
//!
//! ## Usage
//!
//! ```ignore
//! let store = Store::open()?;  // Creates or loads existing
//!
//! // Add data
//! store.add_link(&link)?;
//!
//! // Query data (uses SQLite)
//! let links = store.get_all_links()?;
//! ```

use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::config::Config;
use crate::document::RottDocument;
use crate::document_id::DocumentId;
use crate::models::{Link, Note};
use crate::storage::{AutomergePersistence, SqliteProjection, StorageStats};

/// Unified storage interface for ROTT
///
/// Manages the root Automerge document and keeps SQLite in sync.
pub struct Store {
    /// The root Automerge document (shared for sync)
    doc: Arc<Mutex<RottDocument>>,
    /// Automerge persistence handler
    persistence: AutomergePersistence,
    /// SQLite projection for queries
    projection: SqliteProjection,
    /// Configuration
    config: Config,
}

impl Store {
    /// Open the store, creating a new root document if none exists
    ///
    /// On first run:
    /// - Creates a new Automerge document
    /// - Saves it to disk
    /// - Initializes SQLite
    ///
    /// On subsequent runs:
    /// - Loads existing document
    /// - Rebuilds SQLite projection (ensures consistency)
    pub fn open() -> Result<Self> {
        let config = Config::load().context("Failed to load configuration")?;
        Self::open_with_config(config)
    }

    /// Open the store with a specific configuration
    pub fn open_with_config(config: Config) -> Result<Self> {
        let persistence = AutomergePersistence::new(config.clone());

        // Validate storage is accessible
        persistence
            .validate_storage()
            .context("Storage validation failed")?;

        let mut projection =
            SqliteProjection::open(&config).context("Failed to open SQLite database")?;

        // Load or create the root document (with recovery for corruption)
        let (doc, was_recovered) = persistence
            .load_or_create_with_recovery()
            .context("Failed to load or create root document")?;

        if was_recovered {
            eprintln!(
                "Warning: Document was corrupted and has been recovered. \
                 A backup of the old document has been saved."
            );
        }

        // Rebuild SQLite projection to ensure consistency
        projection
            .project_full(&doc)
            .context("Failed to project document to SQLite")?;

        Ok(Self {
            doc: Arc::new(Mutex::new(doc)),
            persistence,
            projection,
            config,
        })
    }

    /// Get a clone of the shared document handle (for sync)
    pub fn shared_document(&self) -> Arc<Mutex<RottDocument>> {
        Arc::clone(&self.doc)
    }

    /// Get the root document ID
    ///
    /// This ID serves as the user's identity for sync purposes.
    pub fn root_id(&self) -> DocumentId {
        self.doc.blocking_lock().id().clone()
    }

    /// Get the Automerge URL for the root document
    pub fn root_url(&self) -> String {
        self.doc.blocking_lock().url()
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check if this is a new store (just created)
    pub fn is_new(&self) -> bool {
        self.projection.link_count().unwrap_or(0) == 0
    }

    // ==================== Link Operations ====================

    /// Add a new link
    pub fn add_link(&mut self, link: &Link) -> Result<()> {
        self.doc
            .blocking_lock()
            .add_link(link)
            .context("Failed to add link to document")?;
        self.save_and_project()
    }

    /// Update an existing link
    pub fn update_link(&mut self, link: &Link) -> Result<()> {
        self.doc
            .blocking_lock()
            .update_link(link)
            .context("Failed to update link in document")?;
        self.save_and_project()
    }

    /// Delete a link
    pub fn delete_link(&mut self, id: Uuid) -> Result<()> {
        self.doc
            .blocking_lock()
            .delete_link(id)
            .context("Failed to delete link from document")?;
        self.save_and_project()
    }

    /// Get a link by ID (includes notes)
    pub fn get_link(&self, id: Uuid) -> Result<Option<Link>> {
        self.projection
            .get_link(&id.to_string())
            .context("Failed to get link")
    }

    /// Get all links
    pub fn get_all_links(&self) -> Result<Vec<Link>> {
        self.projection
            .get_all_links()
            .context("Failed to get links")
    }

    /// Get links by tag
    pub fn get_links_by_tag(&self, tag: &str) -> Result<Vec<Link>> {
        self.projection
            .get_links_by_tag(tag)
            .context("Failed to get links by tag")
    }

    /// Search links using full-text search
    pub fn search_links(&self, query: &str) -> Result<Vec<Link>> {
        self.projection
            .search_links(query)
            .context("Failed to search links")
    }

    // ==================== Note Operations (via Link) ====================

    /// Add a note to a link
    pub fn add_note_to_link(&mut self, link_id: Uuid, note: &Note) -> Result<()> {
        self.doc
            .blocking_lock()
            .add_note_to_link(link_id, note)
            .context("Failed to add note to link")?;
        self.save_and_project()
    }

    /// Remove a note from a link
    pub fn remove_note_from_link(&mut self, link_id: Uuid, note_id: Uuid) -> Result<()> {
        self.doc
            .blocking_lock()
            .remove_note_from_link(link_id, note_id)
            .context("Failed to remove note from link")?;
        self.save_and_project()
    }

    // ==================== Tag Operations ====================

    /// Get all unique tags
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        self.projection.get_all_tags().context("Failed to get tags")
    }

    /// Get tags with usage counts
    pub fn get_tags_with_counts(&self) -> Result<Vec<(String, i64)>> {
        self.projection
            .get_tags_with_counts()
            .context("Failed to get tag counts")
    }

    // ==================== Stats ====================

    /// Get count of links
    pub fn link_count(&self) -> Result<i64> {
        self.projection
            .link_count()
            .context("Failed to count links")
    }

    /// Get count of notes (across all links)
    pub fn note_count(&self) -> Result<i64> {
        self.projection
            .note_count()
            .context("Failed to count notes")
    }

    // ==================== Advanced ====================

    /// Save the document and update SQLite projection
    ///
    /// Call this after making direct modifications to the document.
    pub fn save_and_project(&mut self) -> Result<()> {
        let mut doc = self.doc.blocking_lock();
        self.persistence
            .save(&mut doc)
            .context("Failed to save document")?;
        self.projection
            .project_full(&doc)
            .context("Failed to project to SQLite")?;
        Ok(())
    }

    /// Force a full rebuild of the SQLite projection
    ///
    /// Useful if SQLite gets out of sync or corrupted, or after sync updates.
    pub fn rebuild_projection(&mut self) -> Result<()> {
        let doc = self.doc.blocking_lock();
        self.projection
            .project_full(&doc)
            .context("Failed to rebuild projection")
    }

    /// Get storage statistics
    pub fn storage_stats(&self) -> StorageStats {
        self.persistence.storage_stats()
    }

    /// Validate that storage is accessible and writable
    pub fn validate_storage(&self) -> Result<()> {
        self.persistence
            .validate_storage()
            .context("Storage validation failed")
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
        }
    }

    #[test]
    fn test_open_creates_new_store() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        let store = Store::open_with_config(config.clone()).unwrap();

        // Root document should be created
        assert!(!store.root_url().is_empty());
        assert!(store.root_url().starts_with("automerge:"));

        // Files should exist
        assert!(config.automerge_path().exists());
        assert!(config.root_doc_id_path().exists());
        assert!(config.sqlite_path().exists());
    }

    #[test]
    fn test_open_loads_existing_store() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Create store and add data
        let original_id;
        {
            let mut store = Store::open_with_config(config.clone()).unwrap();
            original_id = store.root_id();

            let link = Link::new("https://example.com");
            store.add_link(&link).unwrap();
        }

        // Reopen - should load existing data
        let store = Store::open_with_config(config).unwrap();
        assert_eq!(store.root_id(), original_id);
        assert_eq!(store.link_count().unwrap(), 1);
    }

    #[test]
    fn test_root_id_is_stable() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        let id1 = Store::open_with_config(config.clone()).unwrap().root_id();
        let id2 = Store::open_with_config(config.clone()).unwrap().root_id();
        let id3 = Store::open_with_config(config).unwrap().root_id();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_add_and_get_link() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link = Link::new("https://rust-lang.org");
        link.set_title("Rust");
        link.add_tag("programming");

        store.add_link(&link).unwrap();

        let retrieved = store.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.url, "https://rust-lang.org");
        assert_eq!(retrieved.title, "Rust");
        assert!(retrieved.tags.contains(&"programming".to_string()));
    }

    #[test]
    fn test_update_link() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link = Link::new("https://example.com");
        store.add_link(&link).unwrap();

        link.set_title("Updated Title");
        link.add_tag("updated");
        store.update_link(&link).unwrap();

        let retrieved = store.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert!(retrieved.tags.contains(&"updated".to_string()));
    }

    #[test]
    fn test_delete_link() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let link = Link::new("https://example.com");
        store.add_link(&link).unwrap();
        assert_eq!(store.link_count().unwrap(), 1);

        store.delete_link(link.id).unwrap();
        assert_eq!(store.link_count().unwrap(), 0);
        assert!(store.get_link(link.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_links() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        store.add_link(&Link::new("https://one.com")).unwrap();
        store.add_link(&Link::new("https://two.com")).unwrap();
        store.add_link(&Link::new("https://three.com")).unwrap();

        let links = store.get_all_links().unwrap();
        assert_eq!(links.len(), 3);
    }

    #[test]
    fn test_get_links_by_tag() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link1 = Link::new("https://rust-lang.org");
        link1.add_tag("rust");
        store.add_link(&link1).unwrap();

        let mut link2 = Link::new("https://python.org");
        link2.add_tag("python");
        store.add_link(&link2).unwrap();

        let rust_links = store.get_links_by_tag("rust").unwrap();
        assert_eq!(rust_links.len(), 1);
        assert_eq!(rust_links[0].url, "https://rust-lang.org");
    }

    #[test]
    fn test_search_links() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link = Link::new("https://rust-lang.org");
        link.set_title("Rust Programming Language");
        store.add_link(&link).unwrap();

        let results = store.search_links("Programming").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_add_note_to_link() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let link = Link::new("https://example.com");
        let link_id = link.id;
        store.add_link(&link).unwrap();

        let note = Note::new("Great article!");
        store.add_note_to_link(link_id, &note).unwrap();

        let retrieved = store.get_link(link_id).unwrap().unwrap();
        assert_eq!(retrieved.notes.len(), 1);
        assert_eq!(retrieved.notes[0].body, "Great article!");
    }

    #[test]
    fn test_add_note_with_title() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let link = Link::new("https://example.com");
        let link_id = link.id;
        store.add_link(&link).unwrap();

        let note = Note::with_title("Summary", "This is a summary of the article");
        store.add_note_to_link(link_id, &note).unwrap();

        let retrieved = store.get_link(link_id).unwrap().unwrap();
        assert_eq!(retrieved.notes[0].title, Some("Summary".to_string()));
    }

    #[test]
    fn test_remove_note_from_link() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let link = Link::new("https://example.com");
        let link_id = link.id;
        store.add_link(&link).unwrap();

        let note = Note::new("To be removed");
        let note_id = note.id;
        store.add_note_to_link(link_id, &note).unwrap();

        assert_eq!(store.note_count().unwrap(), 1);

        store.remove_note_from_link(link_id, note_id).unwrap();

        assert_eq!(store.note_count().unwrap(), 0);
        let retrieved = store.get_link(link_id).unwrap().unwrap();
        assert!(retrieved.notes.is_empty());
    }

    #[test]
    fn test_link_with_multiple_notes() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let link = Link::new("https://example.com");
        let link_id = link.id;
        store.add_link(&link).unwrap();

        store
            .add_note_to_link(link_id, &Note::new("First note"))
            .unwrap();
        store
            .add_note_to_link(link_id, &Note::new("Second note"))
            .unwrap();
        store
            .add_note_to_link(link_id, &Note::with_title("Third", "With title"))
            .unwrap();

        let retrieved = store.get_link(link_id).unwrap().unwrap();
        assert_eq!(retrieved.notes.len(), 3);
        assert_eq!(store.note_count().unwrap(), 3);
    }

    #[test]
    fn test_get_all_tags() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link = Link::new("https://example.com");
        link.add_tag("web");
        link.add_tag("rust");
        store.add_link(&link).unwrap();

        let mut link2 = Link::new("https://example2.com");
        link2.add_tag("rust");
        link2.add_tag("idea");
        store.add_link(&link2).unwrap();

        let tags = store.get_all_tags().unwrap();
        assert_eq!(tags.len(), 3); // idea, rust, web (alphabetical)
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"web".to_string()));
        assert!(tags.contains(&"idea".to_string()));
    }

    #[test]
    fn test_get_tags_with_counts() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link1 = Link::new("https://example.com");
        link1.add_tag("shared");
        store.add_link(&link1).unwrap();

        let mut link2 = Link::new("https://example2.com");
        link2.add_tag("shared");
        store.add_link(&link2).unwrap();

        let tags = store.get_tags_with_counts().unwrap();
        let shared = tags.iter().find(|(name, _)| name == "shared").unwrap();
        assert_eq!(shared.1, 2);
    }

    #[test]
    fn test_is_new() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        assert!(store.is_new());

        store.add_link(&Link::new("https://example.com")).unwrap();

        assert!(!store.is_new());
    }

    #[test]
    fn test_data_persists_across_reopens() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Create and add data
        {
            let mut store = Store::open_with_config(config.clone()).unwrap();

            let mut link = Link::new("https://persist.com");
            link.set_title("Persistent Link");
            let link_id = link.id;
            store.add_link(&link).unwrap();

            store
                .add_note_to_link(link_id, &Note::new("Persistent note"))
                .unwrap();
        }

        // Reopen and verify
        {
            let store = Store::open_with_config(config).unwrap();

            assert_eq!(store.link_count().unwrap(), 1);
            assert_eq!(store.note_count().unwrap(), 1);

            let links = store.get_all_links().unwrap();
            assert_eq!(links[0].title, "Persistent Link");
            assert_eq!(links[0].notes.len(), 1);
            assert_eq!(links[0].notes[0].body, "Persistent note");
        }
    }

    #[test]
    fn test_rebuild_projection() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        store.add_link(&Link::new("https://example.com")).unwrap();
        assert_eq!(store.link_count().unwrap(), 1);

        // Rebuild should produce same result
        store.rebuild_projection().unwrap();
        assert_eq!(store.link_count().unwrap(), 1);
    }
}
