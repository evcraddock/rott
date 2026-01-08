//! SQLite projection layer
//!
//! Projects Automerge document state to SQLite for fast read queries.
//! SQLite serves as a read-optimized cache; Automerge remains source of truth.
//!
//! ## Architecture
//!
//! - Full projection: Clears and rebuilds all SQLite data from Automerge
//! - Designed for future vector extension compatibility
//!
//! ## Tables
//!
//! - `links` - Link records
//! - `notes` - Note records (children of links)
//! - `tags` - Normalized tag names
//! - `link_tags` - Link-to-tag junction
//! - `link_authors` - Authors for each link
//! - `links_fts` / `notes_fts` - Full-text search (auto-synced via triggers)

use anyhow::{Context, Result};
use rusqlite::{params, Connection, Transaction};

use crate::config::Config;
use crate::document::RottDocument;
use crate::models::{Link, Note};
use crate::storage::schema::{init_schema, needs_init};

/// SQLite projection layer for read-optimized queries
pub struct SqliteProjection {
    conn: Connection,
}

impl SqliteProjection {
    /// Open or create the SQLite database
    pub fn open(config: &Config) -> Result<Self> {
        let path = config.sqlite_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open SQLite database at {:?}", path))?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Initialize schema if needed
        if needs_init(&conn) {
            init_schema(&conn).context("Failed to initialize SQLite schema")?;
        }

        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Project the entire Automerge document to SQLite
    ///
    /// This performs a full rebuild: clears all existing data and
    /// repopulates from the document. Runs in a transaction for atomicity.
    pub fn project_full(&mut self, doc: &RottDocument) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Clear existing data (in correct order for foreign keys)
        clear_all_data(&tx)?;

        // Project links (which includes their notes)
        let links = doc
            .get_all_links()
            .context("Failed to get links from document")?;
        for link in &links {
            insert_link(&tx, link)?;
        }

        tx.commit()?;
        Ok(())
    }

    // ==================== Query Methods ====================

    /// Get all links (without notes - use get_link for full details)
    pub fn get_all_links(&self) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, url, description, created_at, updated_at FROM links ORDER BY created_at DESC",
        )?;

        let link_rows = stmt.query_map([], |row| {
            Ok(LinkRow {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        let mut links = Vec::new();
        for row in link_rows {
            let row = row?;
            let link = self.hydrate_link(row)?;
            links.push(link);
        }

        Ok(links)
    }

    /// Get a link by ID (includes notes)
    pub fn get_link(&self, id: &str) -> Result<Option<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, url, description, created_at, updated_at FROM links WHERE id = ?",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let link_row = LinkRow {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            };
            Ok(Some(self.hydrate_link(link_row)?))
        } else {
            Ok(None)
        }
    }

    /// Get links by tag
    pub fn get_links_by_tag(&self, tag: &str) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT l.id, l.title, l.url, l.description, l.created_at, l.updated_at
            FROM links l
            JOIN link_tags lt ON l.id = lt.link_id
            JOIN tags t ON lt.tag_id = t.id
            WHERE t.name = ?
            ORDER BY l.created_at DESC
            "#,
        )?;

        let link_rows = stmt.query_map(params![tag], |row| {
            Ok(LinkRow {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        let mut links = Vec::new();
        for row in link_rows {
            let row = row?;
            let link = self.hydrate_link(row)?;
            links.push(link);
        }

        Ok(links)
    }

    /// Search links using full-text search
    pub fn search_links(&self, query: &str) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT l.id, l.title, l.url, l.description, l.created_at, l.updated_at
            FROM links l
            JOIN links_fts fts ON l.rowid = fts.rowid
            WHERE links_fts MATCH ?
            ORDER BY rank
            "#,
        )?;

        let link_rows = stmt.query_map(params![query], |row| {
            Ok(LinkRow {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        let mut links = Vec::new();
        for row in link_rows {
            let row = row?;
            let link = self.hydrate_link(row)?;
            links.push(link);
        }

        Ok(links)
    }

    /// Get all unique tags
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM tags ORDER BY name")?;
        let tags = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(tags)
    }

    /// Get tags with usage counts
    pub fn get_tags_with_counts(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT t.name, COUNT(lt.link_id) as count
            FROM tags t
            LEFT JOIN link_tags lt ON t.id = lt.tag_id
            GROUP BY t.id
            ORDER BY count DESC, t.name
            "#,
        )?;

        let tags = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<(String, i64)>, _>>()?;
        Ok(tags)
    }

    /// Get link count
    pub fn link_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))
            .map_err(Into::into)
    }

    /// Get note count
    pub fn note_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .map_err(Into::into)
    }

    // ==================== Private helpers ====================

    /// Hydrate a link with its tags, authors, and notes
    fn hydrate_link(&self, row: LinkRow) -> Result<Link> {
        let tags = self.get_tags_for_link(&row.id)?;
        let authors = self.get_authors_for_link(&row.id)?;
        let notes = self.get_notes_for_link(&row.id)?;

        let id =
            uuid::Uuid::parse_str(&row.id).with_context(|| format!("Invalid UUID: {}", row.id))?;

        let created_at = chrono::DateTime::from_timestamp_millis(row.created_at)
            .unwrap_or_else(chrono::Utc::now);
        let updated_at = chrono::DateTime::from_timestamp_millis(row.updated_at)
            .unwrap_or_else(chrono::Utc::now);

        Ok(Link {
            id,
            title: row.title,
            url: row.url,
            description: row.description,
            author: authors,
            tags,
            created_at,
            updated_at,
            notes,
        })
    }

    fn get_tags_for_link(&self, link_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT t.name FROM tags t
            JOIN link_tags lt ON t.id = lt.tag_id
            WHERE lt.link_id = ?
            ORDER BY t.name
            "#,
        )?;

        let tags = stmt
            .query_map(params![link_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(tags)
    }

    fn get_authors_for_link(&self, link_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT author FROM link_authors WHERE link_id = ? ORDER BY position")?;

        let authors = stmt
            .query_map(params![link_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(authors)
    }

    fn get_notes_for_link(&self, link_id: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, body, created_at FROM notes WHERE link_id = ? ORDER BY created_at",
        )?;

        let notes = stmt
            .query_map(params![link_id], |row| {
                let id: String = row.get(0)?;
                let title: Option<String> = row.get(1)?;
                let body: String = row.get(2)?;
                let created_at: i64 = row.get(3)?;
                Ok((id, title, body, created_at))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        notes
            .into_iter()
            .map(|(id, title, body, created_at)| {
                let id = uuid::Uuid::parse_str(&id)
                    .with_context(|| format!("Invalid note UUID: {}", id))?;
                let created_at = chrono::DateTime::from_timestamp_millis(created_at)
                    .unwrap_or_else(chrono::Utc::now);
                Ok(Note {
                    id,
                    title,
                    body,
                    created_at,
                })
            })
            .collect()
    }
}

// ==================== Internal structs ====================

struct LinkRow {
    id: String,
    title: String,
    url: String,
    description: Option<String>,
    created_at: i64,
    updated_at: i64,
}

// ==================== Transaction helpers ====================

/// Clear all data from tables (preserving schema)
fn clear_all_data(tx: &Transaction) -> Result<()> {
    // Order matters due to foreign keys
    tx.execute("DELETE FROM link_tags", [])?;
    tx.execute("DELETE FROM link_authors", [])?;
    tx.execute("DELETE FROM notes", [])?;
    tx.execute("DELETE FROM links", [])?;
    tx.execute("DELETE FROM tags", [])?;
    Ok(())
}

/// Insert a link and its related data (including notes)
fn insert_link(tx: &Transaction, link: &Link) -> Result<()> {
    // Insert main link record
    tx.execute(
        r#"
        INSERT INTO links (id, title, url, description, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        params![
            link.id.to_string(),
            link.title,
            link.url,
            link.description,
            link.created_at.timestamp_millis(),
            link.updated_at.timestamp_millis(),
        ],
    )?;

    // Insert authors
    for (i, author) in link.author.iter().enumerate() {
        tx.execute(
            "INSERT INTO link_authors (link_id, author, position) VALUES (?, ?, ?)",
            params![link.id.to_string(), author, i as i32],
        )?;
    }

    // Insert tags
    for tag in &link.tags {
        let tag_id = get_or_create_tag(tx, tag)?;
        tx.execute(
            "INSERT INTO link_tags (link_id, tag_id) VALUES (?, ?)",
            params![link.id.to_string(), tag_id],
        )?;
    }

    // Insert notes
    for note in &link.notes {
        tx.execute(
            "INSERT INTO notes (id, link_id, title, body, created_at) VALUES (?, ?, ?, ?, ?)",
            params![
                note.id.to_string(),
                link.id.to_string(),
                note.title,
                note.body,
                note.created_at.timestamp_millis(),
            ],
        )?;
    }

    Ok(())
}

/// Get or create a tag, returning its ID
fn get_or_create_tag(tx: &Transaction, name: &str) -> Result<i64> {
    // Try to get existing tag
    let existing: Option<i64> = tx
        .query_row("SELECT id FROM tags WHERE name = ?", params![name], |row| {
            row.get(0)
        })
        .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    // Create new tag
    tx.execute("INSERT INTO tags (name) VALUES (?)", params![name])?;
    Ok(tx.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_doc() -> RottDocument {
        let mut doc = RottDocument::new();

        // Add some links with notes
        let mut link1 = Link::new("https://rust-lang.org");
        link1.set_title("Rust Programming Language");
        link1.add_tag("programming");
        link1.add_tag("rust");
        link1.set_author(vec!["Mozilla".to_string()]);
        link1.add_note(Note::new("Great language for systems programming"));
        link1.add_note(Note::with_title("Memory Safety", "No null pointers!"));
        doc.add_link(&link1).unwrap();

        let mut link2 = Link::new("https://example.com");
        link2.set_title("Example Site");
        link2.set_description(Some("An example website".to_string()));
        link2.add_tag("example");
        doc.add_link(&link2).unwrap();

        doc
    }

    #[test]
    fn test_project_full() {
        let doc = create_test_doc();
        let mut projection = SqliteProjection::open_in_memory().unwrap();

        projection.project_full(&doc).unwrap();

        assert_eq!(projection.link_count().unwrap(), 2);
        assert_eq!(projection.note_count().unwrap(), 2);
    }

    #[test]
    fn test_get_all_links() {
        let doc = create_test_doc();
        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let links = projection.get_all_links().unwrap();
        assert_eq!(links.len(), 2);

        // Check that tags, authors, and notes are hydrated
        let rust_link = links
            .iter()
            .find(|l| l.url == "https://rust-lang.org")
            .unwrap();
        assert_eq!(rust_link.title, "Rust Programming Language");
        assert!(rust_link.tags.contains(&"rust".to_string()));
        assert!(rust_link.tags.contains(&"programming".to_string()));
        assert_eq!(rust_link.author, vec!["Mozilla"]);
        assert_eq!(rust_link.notes.len(), 2);
    }

    #[test]
    fn test_get_link_by_id() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://test.com");
        link.add_note(Note::new("A test note"));
        let link_id = link.id;
        doc.add_link(&link).unwrap();

        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let found = projection.get_link(&link_id.to_string()).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.url, "https://test.com");
        assert_eq!(found.notes.len(), 1);
        assert_eq!(found.notes[0].body, "A test note");

        let not_found = projection
            .get_link(&uuid::Uuid::new_v4().to_string())
            .unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_links_by_tag() {
        let doc = create_test_doc();
        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let rust_links = projection.get_links_by_tag("rust").unwrap();
        assert_eq!(rust_links.len(), 1);
        assert_eq!(rust_links[0].url, "https://rust-lang.org");

        let example_links = projection.get_links_by_tag("example").unwrap();
        assert_eq!(example_links.len(), 1);

        let no_links = projection.get_links_by_tag("nonexistent").unwrap();
        assert!(no_links.is_empty());
    }

    #[test]
    fn test_search_links() {
        let doc = create_test_doc();
        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        // Search by title
        let results = projection.search_links("Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");

        // Search by description
        let results = projection.search_links("example website").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com");
    }

    #[test]
    fn test_notes_sorted_by_created_at() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://test.com");

        let note1 = Note::new("First note");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let note2 = Note::new("Second note");

        link.add_note(note1);
        link.add_note(note2);
        let link_id = link.id;
        doc.add_link(&link).unwrap();

        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let found = projection.get_link(&link_id.to_string()).unwrap().unwrap();
        assert_eq!(found.notes.len(), 2);
        assert_eq!(found.notes[0].body, "First note");
        assert_eq!(found.notes[1].body, "Second note");
    }

    #[test]
    fn test_note_with_optional_title() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://test.com");

        link.add_note(Note::new("No title"));
        link.add_note(Note::with_title("Has Title", "With title"));
        let link_id = link.id;
        doc.add_link(&link).unwrap();

        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let found = projection.get_link(&link_id.to_string()).unwrap().unwrap();
        assert_eq!(found.notes[0].title, None);
        assert_eq!(found.notes[1].title, Some("Has Title".to_string()));
    }

    #[test]
    fn test_get_all_tags() {
        let doc = create_test_doc();
        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let tags = projection.get_all_tags().unwrap();
        // Tags: example, programming, rust (alphabetical)
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"programming".to_string()));
        assert!(tags.contains(&"example".to_string()));
    }

    #[test]
    fn test_get_tags_with_counts() {
        let doc = create_test_doc();
        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let tags = projection.get_tags_with_counts().unwrap();

        // Each tag is used once
        for (_name, count) in &tags {
            assert_eq!(*count, 1);
        }
    }

    #[test]
    fn test_project_full_replaces_data() {
        let mut doc = RottDocument::new();
        let link = Link::new("https://first.com");
        doc.add_link(&link).unwrap();

        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();
        assert_eq!(projection.link_count().unwrap(), 1);

        // Create new document with different data
        let mut doc2 = RottDocument::new();
        let link2 = Link::new("https://second.com");
        let link3 = Link::new("https://third.com");
        doc2.add_link(&link2).unwrap();
        doc2.add_link(&link3).unwrap();

        // Project replaces all data
        projection.project_full(&doc2).unwrap();
        assert_eq!(projection.link_count().unwrap(), 2);

        let links = projection.get_all_links().unwrap();
        assert!(links.iter().all(|l| l.url != "https://first.com"));
    }

    #[test]
    fn test_empty_document_projection() {
        let doc = RottDocument::new();
        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        assert_eq!(projection.link_count().unwrap(), 0);
        assert_eq!(projection.note_count().unwrap(), 0);
        assert!(projection.get_all_tags().unwrap().is_empty());
    }

    #[test]
    fn test_link_with_multiple_authors() {
        let mut doc = RottDocument::new();
        let mut link = Link::new("https://paper.com");
        link.set_author(vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ]);
        doc.add_link(&link).unwrap();

        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let links = projection.get_all_links().unwrap();
        assert_eq!(links[0].author, vec!["Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn test_special_characters_in_content() {
        let mut doc = RottDocument::new();

        let mut link = Link::new("https://example.com/path?query=value&other=123");
        link.set_title("Test \"quotes\" and 'apostrophes'");
        link.set_description(Some("Description with\nnewlines\tand\ttabs".to_string()));
        link.add_tag("tag-with-dash");
        link.add_tag("tag_with_underscore");
        link.add_note(Note::new("Note with \"special\" characters"));
        doc.add_link(&link).unwrap();

        let mut projection = SqliteProjection::open_in_memory().unwrap();
        projection.project_full(&doc).unwrap();

        let links = projection.get_all_links().unwrap();
        assert_eq!(links[0].title, "Test \"quotes\" and 'apostrophes'");
        assert!(links[0].description.as_ref().unwrap().contains('\n'));
        assert_eq!(links[0].notes[0].body, "Note with \"special\" characters");
    }
}
