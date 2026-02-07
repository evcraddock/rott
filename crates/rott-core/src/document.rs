//! Automerge document handling
//!
//! This module provides the integration between our domain models (Link, Note)
//! and Automerge documents. It handles serialization to/from Automerge format.
//!
//! Document structure:
//! ```text
//! {
//!   schema_version: 2,
//!   root_doc_id: "...",
//!   links: {
//!     "<uuid>": {
//!       id, title, url, description, author, tags, created_at, updated_at,
//!       notes: {
//!         "<uuid>": { id, title, body, created_at },
//!         ...
//!       }
//!     },
//!     ...
//!   }
//! }
//! ```

use automerge::{transaction::Transactable, AutoCommit, ObjType, ReadDoc, ROOT};
use chrono::{DateTime, TimeZone, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::document_id::DocumentId;
use crate::models::{Link, Note};

/// Errors that can occur during document operations
#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field type for {0}")]
    InvalidType(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
}

/// Keys used in the Automerge document structure
mod keys {
    pub const LINKS: &str = "links";
    pub const NOTES: &str = "notes";
    pub const SCHEMA_VERSION: &str = "schema_version";
    pub const ROOT_DOC_ID: &str = "root_doc_id";

    // Link fields
    pub const ID: &str = "id";
    pub const TITLE: &str = "title";
    pub const URL: &str = "url";
    pub const BODY: &str = "body";
    pub const DESCRIPTION: &str = "description";
    pub const AUTHOR: &str = "author";
    pub const TAGS: &str = "tags";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
}

/// Current schema version (bumped for notes-as-children change)
pub const CURRENT_SCHEMA_VERSION: u64 = 2;

/// A ROTT document backed by Automerge
pub struct RottDocument {
    /// The document ID (compatible with automerge-repo)
    id: DocumentId,
    /// The Automerge document
    doc: AutoCommit,
}

impl RottDocument {
    /// Create a new empty document with a random ID
    pub fn new() -> Self {
        Self::with_id(DocumentId::new())
    }

    /// Create a new empty document with a specific ID
    pub fn with_id(id: DocumentId) -> Self {
        let mut doc = AutoCommit::new();

        // Initialize document structure
        doc.put(ROOT, keys::SCHEMA_VERSION, CURRENT_SCHEMA_VERSION)
            .expect("Failed to set schema version");
        doc.put(ROOT, keys::ROOT_DOC_ID, id.to_bs58check())
            .expect("Failed to set root doc ID");
        doc.put_object(ROOT, keys::LINKS, ObjType::Map)
            .expect("Failed to create links map");

        Self { id, doc }
    }

    /// Create an empty document for initial sync (no local changes)
    ///
    /// This creates a document with no local history, intended to receive
    /// the full document state from a sync server. The ID is used to
    /// identify which document to request from the server.
    ///
    /// After syncing, the document will have the server's full history.
    pub fn empty_for_sync(id: DocumentId) -> Self {
        let doc = AutoCommit::new();
        Self { id, doc }
    }

    /// Load a document from Automerge bytes
    pub fn load(bytes: &[u8]) -> Result<Self, DocumentError> {
        let doc = AutoCommit::load(bytes)?;

        // Extract the document ID from the document
        let id_str = match doc.get(ROOT, keys::ROOT_DOC_ID)? {
            Some((value, _)) => value
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| DocumentError::InvalidType("root_doc_id".to_string()))?,
            None => return Err(DocumentError::MissingField("root_doc_id".to_string())),
        };

        let id = DocumentId::from_bs58check(&id_str)
            .map_err(|e| DocumentError::InvalidUuid(e.to_string()))?;

        Ok(Self { id, doc })
    }

    /// Get the document ID
    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    /// Get the Automerge URL for this document
    pub fn url(&self) -> String {
        self.id.to_url()
    }

    /// Save the document to bytes
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Fork the document (for creating a new branch)
    pub fn fork(&mut self) -> Self {
        Self {
            id: self.id,
            doc: self.doc.fork(),
        }
    }

    /// Merge another document into this one
    pub fn merge(&mut self, other: &mut RottDocument) -> Result<(), DocumentError> {
        self.doc.merge(&mut other.doc)?;
        Ok(())
    }

    /// Get the underlying Automerge document (for sync operations)
    pub fn inner(&self) -> &AutoCommit {
        &self.doc
    }

    /// Get the underlying Automerge document mutably
    pub fn inner_mut(&mut self) -> &mut AutoCommit {
        &mut self.doc
    }

    // ==================== Links ====================

    /// Add a new link to the document
    pub fn add_link(&mut self, link: &Link) -> Result<(), DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        let link_id = self
            .doc
            .put_object(&links_id, link.id.to_string(), ObjType::Map)?;

        self.write_link_fields(&link_id, link)?;
        Ok(())
    }

    /// Update an existing link
    pub fn update_link(&mut self, link: &Link) -> Result<(), DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        let link_id = self
            .doc
            .get(&links_id, link.id.to_string())?
            .ok_or_else(|| DocumentError::MissingField(format!("link {}", link.id)))?
            .1;

        self.write_link_fields(&link_id, link)?;
        Ok(())
    }

    /// Delete a link from the document
    pub fn delete_link(&mut self, id: Uuid) -> Result<(), DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        self.doc.delete(&links_id, id.to_string())?;
        Ok(())
    }

    /// Get a link by ID
    pub fn get_link(&self, id: Uuid) -> Result<Option<Link>, DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        match self.doc.get(&links_id, id.to_string())? {
            Some((_, link_id)) => Ok(Some(self.read_link(&link_id, id)?)),
            None => Ok(None),
        }
    }

    /// Get all links
    pub fn get_all_links(&self) -> Result<Vec<Link>, DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        let mut links = Vec::new();
        for key in self.doc.keys(&links_id) {
            let id = Uuid::parse_str(&key).map_err(|_| DocumentError::InvalidUuid(key.clone()))?;
            if let Some((_, link_id)) = self.doc.get(&links_id, &key)? {
                links.push(self.read_link(&link_id, id)?);
            }
        }
        Ok(links)
    }

    /// Get links filtered by tag
    pub fn get_links_by_tag(&self, tag: &str) -> Result<Vec<Link>, DocumentError> {
        let all_links = self.get_all_links()?;
        Ok(all_links
            .into_iter()
            .filter(|link| link.tags.iter().any(|t| t == tag))
            .collect())
    }

    /// Get a link by URL (for duplicate detection)
    ///
    /// Performs a linear scan with basic URL normalization (trailing slash removal,
    /// domain lowercasing). Returns the first match found.
    pub fn get_link_by_url(&self, url: &str) -> Result<Option<Link>, DocumentError> {
        let normalized = normalize_url(url);
        let all_links = self.get_all_links()?;
        Ok(all_links.into_iter().find(|link| {
            let link_normalized = normalize_url(&link.url);
            link_normalized == normalized || link.url == url
        }))
    }

    /// Search links using case-insensitive substring matching
    ///
    /// Searches across title, URL, and description fields.
    pub fn search_links(&self, query: &str) -> Result<Vec<Link>, DocumentError> {
        let query_lower = query.to_lowercase();
        let all_links = self.get_all_links()?;
        Ok(all_links
            .into_iter()
            .filter(|link| {
                link.title.to_lowercase().contains(&query_lower)
                    || link.url.to_lowercase().contains(&query_lower)
                    || link
                        .description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    /// Get tags with usage counts
    pub fn get_tags_with_counts(&self) -> Result<Vec<(String, i64)>, DocumentError> {
        let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for link in self.get_all_links()? {
            for tag in link.tags {
                *counts.entry(tag).or_insert(0) += 1;
            }
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        Ok(result)
    }

    /// Get count of all links
    pub fn link_count(&self) -> Result<usize, DocumentError> {
        Ok(self.get_all_links()?.len())
    }

    /// Get count of all notes across all links
    pub fn note_count(&self) -> Result<usize, DocumentError> {
        Ok(self.get_all_links()?.iter().map(|l| l.notes.len()).sum())
    }

    // ==================== Notes (as children of links) ====================

    /// Add a note to a link
    pub fn add_note_to_link(&mut self, link_id: Uuid, note: &Note) -> Result<(), DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        let link_obj_id = self
            .doc
            .get(&links_id, link_id.to_string())?
            .ok_or_else(|| DocumentError::MissingField(format!("link {}", link_id)))?
            .1;

        // Get or create notes map for this link
        let notes_id = match self.doc.get(&link_obj_id, keys::NOTES)? {
            Some((_, id)) => id,
            None => self
                .doc
                .put_object(&link_obj_id, keys::NOTES, ObjType::Map)?,
        };

        let note_obj_id = self
            .doc
            .put_object(&notes_id, note.id.to_string(), ObjType::Map)?;

        self.write_note_fields(&note_obj_id, note)?;

        // Update link's updated_at
        self.doc.put(
            &link_obj_id,
            keys::UPDATED_AT,
            Utc::now().timestamp_millis(),
        )?;

        Ok(())
    }

    /// Remove a note from a link
    pub fn remove_note_from_link(
        &mut self,
        link_id: Uuid,
        note_id: Uuid,
    ) -> Result<(), DocumentError> {
        let links_id = self
            .doc
            .get(ROOT, keys::LINKS)?
            .ok_or_else(|| DocumentError::MissingField("links".to_string()))?
            .1;

        let link_obj_id = self
            .doc
            .get(&links_id, link_id.to_string())?
            .ok_or_else(|| DocumentError::MissingField(format!("link {}", link_id)))?
            .1;

        let notes_id = self
            .doc
            .get(&link_obj_id, keys::NOTES)?
            .ok_or_else(|| DocumentError::MissingField(format!("notes for link {}", link_id)))?
            .1;

        self.doc.delete(&notes_id, note_id.to_string())?;

        // Update link's updated_at
        self.doc.put(
            &link_obj_id,
            keys::UPDATED_AT,
            Utc::now().timestamp_millis(),
        )?;

        Ok(())
    }

    // ==================== Tags ====================

    /// Get all unique tags from links
    pub fn get_all_tags(&self) -> Result<Vec<String>, DocumentError> {
        let mut tags = std::collections::HashSet::new();

        for link in self.get_all_links()? {
            for tag in link.tags {
                tags.insert(tag);
            }
        }

        let mut tags: Vec<_> = tags.into_iter().collect();
        tags.sort();
        Ok(tags)
    }

    // ==================== Private helpers ====================

    fn write_link_fields(
        &mut self,
        obj_id: &automerge::ObjId,
        link: &Link,
    ) -> Result<(), DocumentError> {
        self.doc.put(obj_id, keys::ID, link.id.to_string())?;
        self.doc.put(obj_id, keys::TITLE, link.title.clone())?;
        self.doc.put(obj_id, keys::URL, link.url.clone())?;

        if let Some(ref desc) = link.description {
            self.doc.put(obj_id, keys::DESCRIPTION, desc.clone())?;
        }

        // Write author array
        let author_id = self.doc.put_object(obj_id, keys::AUTHOR, ObjType::List)?;
        for (i, author) in link.author.iter().enumerate() {
            self.doc.insert(&author_id, i, author.clone())?;
        }

        // Write tags array
        let tags_id = self.doc.put_object(obj_id, keys::TAGS, ObjType::List)?;
        for (i, tag) in link.tags.iter().enumerate() {
            self.doc.insert(&tags_id, i, tag.clone())?;
        }

        self.doc
            .put(obj_id, keys::CREATED_AT, link.created_at.timestamp_millis())?;
        self.doc
            .put(obj_id, keys::UPDATED_AT, link.updated_at.timestamp_millis())?;

        // Write notes map
        let notes_id = self.doc.put_object(obj_id, keys::NOTES, ObjType::Map)?;
        for note in &link.notes {
            let note_obj_id = self
                .doc
                .put_object(&notes_id, note.id.to_string(), ObjType::Map)?;
            self.write_note_fields(&note_obj_id, note)?;
        }

        Ok(())
    }

    fn read_link(&self, obj_id: &automerge::ObjId, id: Uuid) -> Result<Link, DocumentError> {
        let title = self.get_string(obj_id, keys::TITLE)?;
        let url = self.get_string(obj_id, keys::URL)?;
        let description = self.get_optional_string(obj_id, keys::DESCRIPTION)?;
        let author = self.get_string_list(obj_id, keys::AUTHOR)?;
        let tags = self.get_string_list(obj_id, keys::TAGS)?;
        let created_at = self.get_timestamp(obj_id, keys::CREATED_AT)?;
        let updated_at = self.get_timestamp(obj_id, keys::UPDATED_AT)?;

        // Read notes
        let notes = self.read_notes_for_link(obj_id)?;

        Ok(Link {
            id,
            title,
            url,
            description,
            author,
            tags,
            created_at,
            updated_at,
            notes,
        })
    }

    fn read_notes_for_link(
        &self,
        link_obj_id: &automerge::ObjId,
    ) -> Result<Vec<Note>, DocumentError> {
        let notes_id = match self.doc.get(link_obj_id, keys::NOTES)? {
            Some((_, id)) => id,
            None => return Ok(Vec::new()),
        };

        let mut notes = Vec::new();
        for key in self.doc.keys(&notes_id) {
            let id = Uuid::parse_str(&key).map_err(|_| DocumentError::InvalidUuid(key.clone()))?;
            if let Some((_, note_obj_id)) = self.doc.get(&notes_id, &key)? {
                notes.push(self.read_note(&note_obj_id, id)?);
            }
        }

        // Sort by created_at
        notes.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(notes)
    }

    fn write_note_fields(
        &mut self,
        obj_id: &automerge::ObjId,
        note: &Note,
    ) -> Result<(), DocumentError> {
        self.doc.put(obj_id, keys::ID, note.id.to_string())?;

        if let Some(ref title) = note.title {
            self.doc.put(obj_id, keys::TITLE, title.clone())?;
        }

        self.doc.put(obj_id, keys::BODY, note.body.clone())?;
        self.doc
            .put(obj_id, keys::CREATED_AT, note.created_at.timestamp_millis())?;

        Ok(())
    }

    fn read_note(&self, obj_id: &automerge::ObjId, id: Uuid) -> Result<Note, DocumentError> {
        let title = self.get_optional_string(obj_id, keys::TITLE)?;
        let body = self.get_string(obj_id, keys::BODY)?;
        let created_at = self.get_timestamp(obj_id, keys::CREATED_AT)?;

        Ok(Note {
            id,
            title,
            body,
            created_at,
        })
    }

    fn get_string(&self, obj_id: &automerge::ObjId, key: &str) -> Result<String, DocumentError> {
        match self.doc.get(obj_id, key)? {
            Some((value, _)) => value
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| DocumentError::InvalidType(key.to_string())),
            None => Err(DocumentError::MissingField(key.to_string())),
        }
    }

    fn get_optional_string(
        &self,
        obj_id: &automerge::ObjId,
        key: &str,
    ) -> Result<Option<String>, DocumentError> {
        match self.doc.get(obj_id, key)? {
            Some((value, _)) => Ok(value.to_str().map(|s| s.to_string())),
            None => Ok(None),
        }
    }

    fn get_string_list(
        &self,
        obj_id: &automerge::ObjId,
        key: &str,
    ) -> Result<Vec<String>, DocumentError> {
        match self.doc.get(obj_id, key)? {
            Some((_, list_id)) => {
                let mut result = Vec::new();
                let len = self.doc.length(&list_id);
                for i in 0..len {
                    if let Some((value, _)) = self.doc.get(&list_id, i)? {
                        if let Some(s) = value.to_str() {
                            result.push(s.to_string());
                        }
                    }
                }
                Ok(result)
            }
            None => Ok(Vec::new()),
        }
    }

    fn get_timestamp(
        &self,
        obj_id: &automerge::ObjId,
        key: &str,
    ) -> Result<DateTime<Utc>, DocumentError> {
        match self.doc.get(obj_id, key)? {
            Some((value, _)) => {
                let millis = value
                    .to_i64()
                    .ok_or_else(|| DocumentError::InvalidType(key.to_string()))?;
                Utc.timestamp_millis_opt(millis)
                    .single()
                    .ok_or_else(|| DocumentError::InvalidTimestamp(millis))
            }
            None => Err(DocumentError::MissingField(key.to_string())),
        }
    }
}

impl Default for RottDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize a URL for duplicate detection
///
/// - Removes trailing slashes (except for root path)
/// - Lowercases the domain portion
fn normalize_url(url: &str) -> String {
    let mut normalized = url.trim().to_string();

    // Remove trailing slash (but not for root path)
    if normalized.ends_with('/') && normalized.matches('/').count() > 3 {
        normalized.pop();
    }

    // Try to lowercase just the domain part
    if let Some(idx) = normalized.find("://") {
        let (scheme, rest) = normalized.split_at(idx + 3);
        if let Some(path_idx) = rest.find('/') {
            let (domain, path) = rest.split_at(path_idx);
            normalized = format!("{}{}{}", scheme, domain.to_lowercase(), path);
        } else {
            normalized = format!("{}{}", scheme, rest.to_lowercase());
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc = RottDocument::new();
        assert!(doc.get_all_links().unwrap().is_empty());
    }

    #[test]
    fn test_add_and_get_link() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://example.com");
        link.set_title("Example Site");
        link.add_tag("test");

        doc.add_link(&link).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.id, link.id);
        assert_eq!(retrieved.title, "Example Site");
        assert_eq!(retrieved.url, "https://example.com");
        assert_eq!(retrieved.tags, vec!["test"]);
        assert!(retrieved.notes.is_empty());
    }

    #[test]
    fn test_update_link() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        link.set_title("Updated Title");
        link.add_tag("updated");
        doc.update_link(&link).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert!(retrieved.tags.contains(&"updated".to_string()));
    }

    #[test]
    fn test_delete_link() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        doc.delete_link(link.id).unwrap();

        assert!(doc.get_link(link.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_links() {
        let mut doc = RottDocument::new();
        let link1 = Link::new("https://example1.com");
        let link2 = Link::new("https://example2.com");

        doc.add_link(&link1).unwrap();
        doc.add_link(&link2).unwrap();

        let links = doc.get_all_links().unwrap();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_get_links_by_tag() {
        let mut doc = RottDocument::new();
        let mut link1 = Link::new("https://rust-lang.org");
        link1.add_tag("rust");
        let mut link2 = Link::new("https://python.org");
        link2.add_tag("python");

        doc.add_link(&link1).unwrap();
        doc.add_link(&link2).unwrap();

        let rust_links = doc.get_links_by_tag("rust").unwrap();
        assert_eq!(rust_links.len(), 1);
        assert_eq!(rust_links[0].url, "https://rust-lang.org");
    }

    #[test]
    fn test_add_note_to_link() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        let note = Note::new("Great article!");
        doc.add_note_to_link(link.id, &note).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.notes.len(), 1);
        assert_eq!(retrieved.notes[0].body, "Great article!");
    }

    #[test]
    fn test_add_note_with_title() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        let note = Note::with_title("Summary", "This article covers...");
        doc.add_note_to_link(link.id, &note).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.notes.len(), 1);
        assert_eq!(retrieved.notes[0].title, Some("Summary".to_string()));
        assert_eq!(retrieved.notes[0].body, "This article covers...");
    }

    #[test]
    fn test_remove_note_from_link() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        let note = Note::new("To be removed");
        let note_id = note.id;
        doc.add_note_to_link(link.id, &note).unwrap();

        doc.remove_note_from_link(link.id, note_id).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert!(retrieved.notes.is_empty());
    }

    #[test]
    fn test_multiple_notes_sorted() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        let note1 = Note::new("First note");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let note2 = Note::new("Second note");

        doc.add_note_to_link(link.id, &note1).unwrap();
        doc.add_note_to_link(link.id, &note2).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.notes.len(), 2);
        assert_eq!(retrieved.notes[0].body, "First note");
        assert_eq!(retrieved.notes[1].body, "Second note");
    }

    #[test]
    fn test_link_with_notes_roundtrip() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://example.com");
        link.add_note(Note::new("Inline note"));
        doc.add_link(&link).unwrap();

        let retrieved = doc.get_link(link.id).unwrap().unwrap();
        assert_eq!(retrieved.notes.len(), 1);
        assert_eq!(retrieved.notes[0].body, "Inline note");
    }

    #[test]
    fn test_get_all_tags() {
        let mut doc = RottDocument::new();

        let mut link1 = Link::new("https://example.com");
        link1.add_tag("web");
        link1.add_tag("rust");

        let mut link2 = Link::new("https://example2.com");
        link2.add_tag("rust");
        link2.add_tag("idea");

        doc.add_link(&link1).unwrap();
        doc.add_link(&link2).unwrap();

        let tags = doc.get_all_tags().unwrap();
        assert_eq!(tags, vec!["idea", "rust", "web"]);
    }

    #[test]
    fn test_save_and_load() {
        let mut doc = RottDocument::new();
        let original_id = *doc.id();
        let mut link = Link::new("https://example.com");
        link.add_note(Note::new("A note"));
        doc.add_link(&link).unwrap();

        let bytes = doc.save();

        let loaded = RottDocument::load(&bytes).unwrap();
        assert_eq!(*loaded.id(), original_id);
        let links = loaded.get_all_links().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com");
        assert_eq!(links[0].notes.len(), 1);
    }

    #[test]
    fn test_document_id_and_url() {
        let doc = RottDocument::new();
        let url = doc.url();
        assert!(url.starts_with("automerge:"));

        // URL should be parseable back to the same ID
        let parsed_id = crate::DocumentId::from_url(&url).unwrap();
        assert_eq!(*doc.id(), parsed_id);
    }

    #[test]
    fn test_merge_documents() {
        let mut doc1 = RottDocument::new();
        let mut doc2 = doc1.fork();

        let link1 = Link::new("https://example1.com");
        let link2 = Link::new("https://example2.com");

        doc1.add_link(&link1).unwrap();
        doc2.add_link(&link2).unwrap();

        doc1.merge(&mut doc2).unwrap();

        let links = doc1.get_all_links().unwrap();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_get_link_by_url_found() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://rust-lang.org");
        link.set_title("Rust");
        doc.add_link(&link).unwrap();

        let found = doc.get_link_by_url("https://rust-lang.org").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Rust");
    }

    #[test]
    fn test_get_link_by_url_not_found() {
        let mut doc = RottDocument::new();
        doc.add_link(&Link::new("https://rust-lang.org")).unwrap();

        let found = doc.get_link_by_url("https://not-exists.com").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_get_link_by_url_normalized() {
        let mut doc = RottDocument::new();
        doc.add_link(&Link::new("https://Example.COM/path/"))
            .unwrap();

        // Should match with different casing on domain
        let found = doc.get_link_by_url("https://example.com/path/").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_search_links_by_title() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://rust-lang.org");
        link.set_title("Rust Programming Language");
        doc.add_link(&link).unwrap();

        let results = doc.search_links("Programming").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");
    }

    #[test]
    fn test_search_links_by_description() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://example.com");
        link.set_title("Example");
        link.set_description(Some("An example website for testing".to_string()));
        doc.add_link(&link).unwrap();

        let results = doc.search_links("testing").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_links_by_url() {
        let mut doc = RottDocument::new();
        doc.add_link(&Link::new("https://rust-lang.org")).unwrap();
        doc.add_link(&Link::new("https://python.org")).unwrap();

        let results = doc.search_links("rust").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_links_no_results() {
        let mut doc = RottDocument::new();
        doc.add_link(&Link::new("https://example.com")).unwrap();

        let results = doc.search_links("nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_links_case_insensitive() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://example.com");
        link.set_title("Rust Programming");
        doc.add_link(&link).unwrap();

        let results = doc.search_links("rust programming").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_tags_with_counts() {
        let mut doc = RottDocument::new();

        let mut link1 = Link::new("https://one.com");
        link1.add_tag("rust");
        link1.add_tag("web");
        doc.add_link(&link1).unwrap();

        let mut link2 = Link::new("https://two.com");
        link2.add_tag("rust");
        doc.add_link(&link2).unwrap();

        let counts = doc.get_tags_with_counts().unwrap();
        let rust_count = counts.iter().find(|(name, _)| name == "rust").unwrap();
        assert_eq!(rust_count.1, 2);

        let web_count = counts.iter().find(|(name, _)| name == "web").unwrap();
        assert_eq!(web_count.1, 1);

        // rust should come first (higher count)
        assert_eq!(counts[0].0, "rust");
    }

    #[test]
    fn test_link_count() {
        let mut doc = RottDocument::new();
        assert_eq!(doc.link_count().unwrap(), 0);

        doc.add_link(&Link::new("https://one.com")).unwrap();
        doc.add_link(&Link::new("https://two.com")).unwrap();
        assert_eq!(doc.link_count().unwrap(), 2);
    }

    #[test]
    fn test_note_count() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://example.com");
        let link_id = link.id;
        doc.add_link(&link).unwrap();

        assert_eq!(doc.note_count().unwrap(), 0);

        doc.add_note_to_link(link_id, &Note::new("Note 1")).unwrap();
        doc.add_note_to_link(link_id, &Note::new("Note 2")).unwrap();
        assert_eq!(doc.note_count().unwrap(), 2);
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            super::normalize_url("https://Example.COM/path"),
            "https://example.com/path"
        );
        assert_eq!(
            super::normalize_url("https://example.com/Path/Case"),
            "https://example.com/Path/Case"
        );
        // Trailing slash removed when there's a path
        assert_eq!(
            super::normalize_url("https://example.com/path/"),
            "https://example.com/path"
        );
    }
}
