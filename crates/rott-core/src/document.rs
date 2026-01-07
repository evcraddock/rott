//! Automerge document handling
//!
//! This module provides the integration between our domain models (Link, Note)
//! and Automerge documents. It handles serialization to/from Automerge format.

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

    // Link/Note fields
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

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u64 = 1;

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
        doc.put_object(ROOT, keys::NOTES, ObjType::Map)
            .expect("Failed to create notes map");

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

    // ==================== Notes ====================

    /// Add a new note to the document
    pub fn add_note(&mut self, note: &Note) -> Result<(), DocumentError> {
        let notes_id = self
            .doc
            .get(ROOT, keys::NOTES)?
            .ok_or_else(|| DocumentError::MissingField("notes".to_string()))?
            .1;

        let note_id = self
            .doc
            .put_object(&notes_id, note.id.to_string(), ObjType::Map)?;

        self.write_note_fields(&note_id, note)?;
        Ok(())
    }

    /// Update an existing note
    pub fn update_note(&mut self, note: &Note) -> Result<(), DocumentError> {
        let notes_id = self
            .doc
            .get(ROOT, keys::NOTES)?
            .ok_or_else(|| DocumentError::MissingField("notes".to_string()))?
            .1;

        let note_id = self
            .doc
            .get(&notes_id, note.id.to_string())?
            .ok_or_else(|| DocumentError::MissingField(format!("note {}", note.id)))?
            .1;

        self.write_note_fields(&note_id, note)?;
        Ok(())
    }

    /// Delete a note from the document
    pub fn delete_note(&mut self, id: Uuid) -> Result<(), DocumentError> {
        let notes_id = self
            .doc
            .get(ROOT, keys::NOTES)?
            .ok_or_else(|| DocumentError::MissingField("notes".to_string()))?
            .1;

        self.doc.delete(&notes_id, id.to_string())?;
        Ok(())
    }

    /// Get a note by ID
    pub fn get_note(&self, id: Uuid) -> Result<Option<Note>, DocumentError> {
        let notes_id = self
            .doc
            .get(ROOT, keys::NOTES)?
            .ok_or_else(|| DocumentError::MissingField("notes".to_string()))?
            .1;

        match self.doc.get(&notes_id, id.to_string())? {
            Some((_, note_id)) => Ok(Some(self.read_note(&note_id, id)?)),
            None => Ok(None),
        }
    }

    /// Get all notes
    pub fn get_all_notes(&self) -> Result<Vec<Note>, DocumentError> {
        let notes_id = self
            .doc
            .get(ROOT, keys::NOTES)?
            .ok_or_else(|| DocumentError::MissingField("notes".to_string()))?
            .1;

        let mut notes = Vec::new();
        for key in self.doc.keys(&notes_id) {
            let id = Uuid::parse_str(&key).map_err(|_| DocumentError::InvalidUuid(key.clone()))?;
            if let Some((_, note_id)) = self.doc.get(&notes_id, &key)? {
                notes.push(self.read_note(&note_id, id)?);
            }
        }
        Ok(notes)
    }

    /// Get notes filtered by tag
    pub fn get_notes_by_tag(&self, tag: &str) -> Result<Vec<Note>, DocumentError> {
        let all_notes = self.get_all_notes()?;
        Ok(all_notes
            .into_iter()
            .filter(|note| note.tags.iter().any(|t| t == tag))
            .collect())
    }

    // ==================== Tags ====================

    /// Get all unique tags from links and notes
    pub fn get_all_tags(&self) -> Result<Vec<String>, DocumentError> {
        let mut tags = std::collections::HashSet::new();

        for link in self.get_all_links()? {
            for tag in link.tags {
                tags.insert(tag);
            }
        }

        for note in self.get_all_notes()? {
            for tag in note.tags {
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

        Ok(Link {
            id,
            title,
            url,
            description,
            author,
            tags,
            created_at,
            updated_at,
        })
    }

    fn write_note_fields(
        &mut self,
        obj_id: &automerge::ObjId,
        note: &Note,
    ) -> Result<(), DocumentError> {
        self.doc.put(obj_id, keys::ID, note.id.to_string())?;
        self.doc.put(obj_id, keys::TITLE, note.title.clone())?;
        self.doc.put(obj_id, keys::BODY, note.body.clone())?;

        // Write tags array
        let tags_id = self.doc.put_object(obj_id, keys::TAGS, ObjType::List)?;
        for (i, tag) in note.tags.iter().enumerate() {
            self.doc.insert(&tags_id, i, tag.clone())?;
        }

        self.doc
            .put(obj_id, keys::CREATED_AT, note.created_at.timestamp_millis())?;
        self.doc
            .put(obj_id, keys::UPDATED_AT, note.updated_at.timestamp_millis())?;

        Ok(())
    }

    fn read_note(&self, obj_id: &automerge::ObjId, id: Uuid) -> Result<Note, DocumentError> {
        let title = self.get_string(obj_id, keys::TITLE)?;
        let body = self.get_string(obj_id, keys::BODY)?;
        let tags = self.get_string_list(obj_id, keys::TAGS)?;
        let created_at = self.get_timestamp(obj_id, keys::CREATED_AT)?;
        let updated_at = self.get_timestamp(obj_id, keys::UPDATED_AT)?;

        Ok(Note {
            id,
            title,
            body,
            tags,
            created_at,
            updated_at,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc = RottDocument::new();
        assert!(doc.get_all_links().unwrap().is_empty());
        assert!(doc.get_all_notes().unwrap().is_empty());
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
    fn test_add_and_get_note() {
        let mut doc = RottDocument::new();
        let mut note = Note::new("Test Note");
        note.set_body("This is the body");
        note.add_tag("idea");

        doc.add_note(&note).unwrap();

        let retrieved = doc.get_note(note.id).unwrap().unwrap();
        assert_eq!(retrieved.id, note.id);
        assert_eq!(retrieved.title, "Test Note");
        assert_eq!(retrieved.body, "This is the body");
        assert_eq!(retrieved.tags, vec!["idea"]);
    }

    #[test]
    fn test_update_note() {
        let mut doc = RottDocument::new();
        let mut note = Note::new("Test Note");
        doc.add_note(&note).unwrap();

        note.set_title("Updated Title");
        note.set_body("Updated body");
        note.add_tag("updated");
        doc.update_note(&note).unwrap();

        let retrieved = doc.get_note(note.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(retrieved.body, "Updated body");
        assert!(retrieved.tags.contains(&"updated".to_string()));
    }

    #[test]
    fn test_delete_note() {
        let mut doc = RottDocument::new();
        let note = Note::new("Test Note");
        doc.add_note(&note).unwrap();

        doc.delete_note(note.id).unwrap();

        assert!(doc.get_note(note.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_notes() {
        let mut doc = RottDocument::new();
        let note1 = Note::new("Note 1");
        let note2 = Note::new("Note 2");

        doc.add_note(&note1).unwrap();
        doc.add_note(&note2).unwrap();

        let notes = doc.get_all_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_get_notes_by_tag() {
        let mut doc = RottDocument::new();
        let mut note1 = Note::new("Rust Note");
        note1.add_tag("rust");
        let mut note2 = Note::new("Python Note");
        note2.add_tag("python");

        doc.add_note(&note1).unwrap();
        doc.add_note(&note2).unwrap();

        let rust_notes = doc.get_notes_by_tag("rust").unwrap();
        assert_eq!(rust_notes.len(), 1);
        assert_eq!(rust_notes[0].title, "Rust Note");
    }

    #[test]
    fn test_get_all_tags() {
        let mut doc = RottDocument::new();

        let mut link = Link::new("https://example.com");
        link.add_tag("web");
        link.add_tag("rust");

        let mut note = Note::new("Note");
        note.add_tag("rust");
        note.add_tag("idea");

        doc.add_link(&link).unwrap();
        doc.add_note(&note).unwrap();

        let tags = doc.get_all_tags().unwrap();
        assert_eq!(tags, vec!["idea", "rust", "web"]);
    }

    #[test]
    fn test_save_and_load() {
        let mut doc = RottDocument::new();
        let original_id = *doc.id();
        let link = Link::new("https://example.com");
        doc.add_link(&link).unwrap();

        let bytes = doc.save();

        let loaded = RottDocument::load(&bytes).unwrap();
        assert_eq!(*loaded.id(), original_id);
        let links = loaded.get_all_links().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com");
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
}
