//! Data models for ROTT
//!
//! Defines the core data structures: Link, Note, and Tag.
//! These models are designed to work with Automerge for CRDT-based sync.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A saved link with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    /// Unique identifier
    pub id: Uuid,
    /// Display title (often fetched from page metadata)
    pub title: String,
    /// The URL
    pub url: String,
    /// Optional description
    pub description: Option<String>,
    /// Author(s) of the linked content
    pub author: Vec<String>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// When this link was created
    pub created_at: DateTime<Utc>,
    /// When this link was last updated
    pub updated_at: DateTime<Utc>,
}

impl Link {
    /// Create a new link with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: url.clone(),
            url,
            description: None,
            author: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a link with a specific ID (for loading from storage)
    pub fn with_id(id: Uuid, url: impl Into<String>) -> Self {
        let url = url.into();
        let now = Utc::now();
        Self {
            id,
            title: url.clone(),
            url,
            description: None,
            author: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    /// Update the description
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Set the authors
    pub fn set_author(&mut self, author: Vec<String>) {
        self.author = author;
        self.updated_at = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Set all tags (replacing existing)
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }
}

/// A text note
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    /// Unique identifier
    pub id: Uuid,
    /// Note title
    pub title: String,
    /// Note body content
    pub body: String,
    /// Tags for organization
    pub tags: Vec<String>,
    /// When this note was created
    pub created_at: DateTime<Utc>,
    /// When this note was last updated
    pub updated_at: DateTime<Utc>,
}

impl Note {
    /// Create a new note with the given title
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            body: String::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a note with a specific ID (for loading from storage)
    pub fn with_id(id: Uuid, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            title: title.into(),
            body: String::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    /// Update the body
    pub fn set_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
        self.updated_at = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Set all tags (replacing existing)
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }
}

/// A tag for organizing links and notes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag(pub String);

impl Tag {
    /// Create a new tag
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the tag name
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_new() {
        let link = Link::new("https://example.com");
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, "https://example.com");
        assert!(link.tags.is_empty());
        assert!(link.author.is_empty());
        assert!(link.description.is_none());
    }

    #[test]
    fn test_link_with_id() {
        let id = Uuid::new_v4();
        let link = Link::with_id(id, "https://example.com");
        assert_eq!(link.id, id);
        assert_eq!(link.url, "https://example.com");
    }

    #[test]
    fn test_link_set_title() {
        let mut link = Link::new("https://example.com");
        let original_updated = link.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        link.set_title("Example Site");
        assert_eq!(link.title, "Example Site");
        assert!(link.updated_at > original_updated);
    }

    #[test]
    fn test_link_tags() {
        let mut link = Link::new("https://example.com");
        link.add_tag("rust");
        link.add_tag("programming");
        assert_eq!(link.tags, vec!["rust", "programming"]);

        // Adding duplicate should not add again
        link.add_tag("rust");
        assert_eq!(link.tags.len(), 2);

        link.remove_tag("rust");
        assert_eq!(link.tags, vec!["programming"]);
    }

    #[test]
    fn test_note_new() {
        let note = Note::new("Test Note");
        assert_eq!(note.title, "Test Note");
        assert!(note.body.is_empty());
        assert!(note.tags.is_empty());
    }

    #[test]
    fn test_note_with_id() {
        let id = Uuid::new_v4();
        let note = Note::with_id(id, "Test Note");
        assert_eq!(note.id, id);
        assert_eq!(note.title, "Test Note");
    }

    #[test]
    fn test_note_set_body() {
        let mut note = Note::new("Test Note");
        note.set_body("This is the note content.");
        assert_eq!(note.body, "This is the note content.");
    }

    #[test]
    fn test_note_tags() {
        let mut note = Note::new("Test Note");
        note.add_tag("idea");
        note.add_tag("project");
        assert_eq!(note.tags, vec!["idea", "project"]);

        note.set_tags(vec!["new-tag".to_string()]);
        assert_eq!(note.tags, vec!["new-tag"]);
    }

    #[test]
    fn test_tag_display() {
        let tag = Tag::new("rust");
        assert_eq!(format!("{}", tag), "rust");
        assert_eq!(tag.name(), "rust");
    }

    #[test]
    fn test_tag_from() {
        let tag1: Tag = "rust".into();
        let tag2: Tag = String::from("rust").into();
        assert_eq!(tag1, tag2);
    }

    #[test]
    fn test_link_serialization() {
        let link = Link::new("https://example.com");
        let json = serde_json::to_string(&link).unwrap();
        let deserialized: Link = serde_json::from_str(&json).unwrap();
        assert_eq!(link, deserialized);
    }

    #[test]
    fn test_note_serialization() {
        let mut note = Note::new("Test Note");
        note.set_body("Content");
        note.add_tag("test");
        let json = serde_json::to_string(&note).unwrap();
        let deserialized: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(note, deserialized);
    }
}
