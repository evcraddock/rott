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

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::config::Config;
use crate::document::RottDocument;
use crate::document_id::DocumentId;
use crate::models::{Link, Note};
use crate::storage::{AutomergePersistence, SqliteProjection};

/// Unified storage interface for ROTT
///
/// Manages the root Automerge document and keeps SQLite in sync.
pub struct Store {
    /// The root Automerge document
    doc: RottDocument,
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
        let mut projection =
            SqliteProjection::open(&config).context("Failed to open SQLite database")?;

        // Load or create the root document
        let doc = persistence
            .load_or_create()
            .context("Failed to load or create root document")?;

        // Rebuild SQLite projection to ensure consistency
        projection
            .project_full(&doc)
            .context("Failed to project document to SQLite")?;

        Ok(Self {
            doc,
            persistence,
            projection,
            config,
        })
    }

    /// Get the root document ID
    ///
    /// This ID serves as the user's identity for sync purposes.
    pub fn root_id(&self) -> &DocumentId {
        self.doc.id()
    }

    /// Get the Automerge URL for the root document
    pub fn root_url(&self) -> String {
        self.doc.url()
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check if this is a new store (just created)
    pub fn is_new(&self) -> bool {
        self.projection.link_count().unwrap_or(0) == 0
            && self.projection.note_count().unwrap_or(0) == 0
    }

    // ==================== Link Operations ====================

    /// Add a new link
    pub fn add_link(&mut self, link: &Link) -> Result<()> {
        self.doc
            .add_link(link)
            .context("Failed to add link to document")?;
        self.save_and_project()
    }

    /// Update an existing link
    pub fn update_link(&mut self, link: &Link) -> Result<()> {
        self.doc
            .update_link(link)
            .context("Failed to update link in document")?;
        self.save_and_project()
    }

    /// Delete a link
    pub fn delete_link(&mut self, id: Uuid) -> Result<()> {
        self.doc
            .delete_link(id)
            .context("Failed to delete link from document")?;
        self.save_and_project()
    }

    /// Get a link by ID
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

    // ==================== Note Operations ====================

    /// Add a new note
    pub fn add_note(&mut self, note: &Note) -> Result<()> {
        self.doc
            .add_note(note)
            .context("Failed to add note to document")?;
        self.save_and_project()
    }

    /// Update an existing note
    pub fn update_note(&mut self, note: &Note) -> Result<()> {
        self.doc
            .update_note(note)
            .context("Failed to update note in document")?;
        self.save_and_project()
    }

    /// Delete a note
    pub fn delete_note(&mut self, id: Uuid) -> Result<()> {
        self.doc
            .delete_note(id)
            .context("Failed to delete note from document")?;
        self.save_and_project()
    }

    /// Get a note by ID
    pub fn get_note(&self, id: Uuid) -> Result<Option<Note>> {
        self.projection
            .get_note(&id.to_string())
            .context("Failed to get note")
    }

    /// Get all notes
    pub fn get_all_notes(&self) -> Result<Vec<Note>> {
        self.projection
            .get_all_notes()
            .context("Failed to get notes")
    }

    /// Get notes by tag
    pub fn get_notes_by_tag(&self, tag: &str) -> Result<Vec<Note>> {
        self.projection
            .get_notes_by_tag(tag)
            .context("Failed to get notes by tag")
    }

    /// Search notes using full-text search
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        self.projection
            .search_notes(query)
            .context("Failed to search notes")
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

    /// Get count of notes
    pub fn note_count(&self) -> Result<i64> {
        self.projection
            .note_count()
            .context("Failed to count notes")
    }

    // ==================== Advanced ====================

    /// Get access to the underlying Automerge document (for sync)
    pub fn document(&self) -> &RottDocument {
        &self.doc
    }

    /// Get mutable access to the underlying Automerge document
    ///
    /// After modifying, call `save_and_project()` to persist changes.
    pub fn document_mut(&mut self) -> &mut RottDocument {
        &mut self.doc
    }

    /// Save the document and update SQLite projection
    ///
    /// Call this after making direct modifications to the document.
    pub fn save_and_project(&mut self) -> Result<()> {
        self.persistence
            .save(&mut self.doc)
            .context("Failed to save document")?;
        self.projection
            .project_full(&self.doc)
            .context("Failed to project to SQLite")?;
        Ok(())
    }

    /// Force a full rebuild of the SQLite projection
    ///
    /// Useful if SQLite gets out of sync or corrupted.
    pub fn rebuild_projection(&mut self) -> Result<()> {
        self.projection
            .project_full(&self.doc)
            .context("Failed to rebuild projection")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn test_config(temp_dir: &TempDir) -> Config {
        Config {
            data_dir: temp_dir.path().to_path_buf(),
            sync_url: None,
            sync_enabled: false,
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
            original_id = *store.root_id();

            let link = Link::new("https://example.com");
            store.add_link(&link).unwrap();
        }

        // Reopen - should load existing data
        let store = Store::open_with_config(config).unwrap();
        assert_eq!(*store.root_id(), original_id);
        assert_eq!(store.link_count().unwrap(), 1);
    }

    #[test]
    fn test_root_id_is_stable() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        let id1 = Store::open_with_config(config.clone())
            .unwrap()
            .root_id()
            .clone();
        let id2 = Store::open_with_config(config.clone())
            .unwrap()
            .root_id()
            .clone();
        let id3 = Store::open_with_config(config).unwrap().root_id().clone();

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
    fn test_add_and_get_note() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut note = Note::new("Test Note");
        note.set_body("This is the body");
        note.add_tag("idea");

        store.add_note(&note).unwrap();

        let retrieved = store.get_note(note.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Note");
        assert_eq!(retrieved.body, "This is the body");
        assert!(retrieved.tags.contains(&"idea".to_string()));
    }

    #[test]
    fn test_update_note() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut note = Note::new("Test Note");
        store.add_note(&note).unwrap();

        note.set_title("Updated Title");
        note.set_body("Updated body");
        store.update_note(&note).unwrap();

        let retrieved = store.get_note(note.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(retrieved.body, "Updated body");
    }

    #[test]
    fn test_delete_note() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let note = Note::new("Test Note");
        store.add_note(&note).unwrap();
        assert_eq!(store.note_count().unwrap(), 1);

        store.delete_note(note.id).unwrap();
        assert_eq!(store.note_count().unwrap(), 0);
    }

    #[test]
    fn test_get_all_tags() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open_with_config(test_config(&temp_dir)).unwrap();

        let mut link = Link::new("https://example.com");
        link.add_tag("web");
        link.add_tag("rust");
        store.add_link(&link).unwrap();

        let mut note = Note::new("Note");
        note.add_tag("rust");
        note.add_tag("idea");
        store.add_note(&note).unwrap();

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

        let mut link = Link::new("https://example.com");
        link.add_tag("shared");
        store.add_link(&link).unwrap();

        let mut note = Note::new("Note");
        note.add_tag("shared");
        store.add_note(&note).unwrap();

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
            store.add_link(&link).unwrap();

            let mut note = Note::new("Persistent Note");
            note.set_body("Body content");
            store.add_note(&note).unwrap();
        }

        // Reopen and verify
        {
            let store = Store::open_with_config(config).unwrap();

            assert_eq!(store.link_count().unwrap(), 1);
            assert_eq!(store.note_count().unwrap(), 1);

            let links = store.get_all_links().unwrap();
            assert_eq!(links[0].title, "Persistent Link");

            let notes = store.get_all_notes().unwrap();
            assert_eq!(notes[0].body, "Body content");
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
