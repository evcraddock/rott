//! SQLite schema for the read-optimized projection layer
//!
//! This schema is designed for fast queries. The Automerge document
//! remains the source of truth; this SQLite database is rebuilt from it.

use rusqlite::{Connection, Result};

/// Current schema version for migrations
pub const SCHEMA_VERSION: i32 = 1;

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Schema version tracking
        CREATE TABLE IF NOT EXISTS schema_info (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Links table
        CREATE TABLE IF NOT EXISTS links (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            description TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- Notes table
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- Authors for links (one-to-many)
        CREATE TABLE IF NOT EXISTS link_authors (
            link_id TEXT NOT NULL,
            author TEXT NOT NULL,
            position INTEGER NOT NULL,
            PRIMARY KEY (link_id, position),
            FOREIGN KEY (link_id) REFERENCES links(id) ON DELETE CASCADE
        );

        -- Tags table (normalized)
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL
        );

        -- Link-tag junction table (many-to-many)
        CREATE TABLE IF NOT EXISTS link_tags (
            link_id TEXT NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (link_id, tag_id),
            FOREIGN KEY (link_id) REFERENCES links(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        -- Note-tag junction table (many-to-many)
        CREATE TABLE IF NOT EXISTS note_tags (
            note_id TEXT NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (note_id, tag_id),
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        -- Indexes for common query patterns

        -- Query links by URL (for duplicate detection)
        CREATE INDEX IF NOT EXISTS idx_links_url ON links(url);

        -- Query by creation date (for sorting/filtering)
        CREATE INDEX IF NOT EXISTS idx_links_created_at ON links(created_at);
        CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at);

        -- Query by update date
        CREATE INDEX IF NOT EXISTS idx_links_updated_at ON links(updated_at);
        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at);

        -- Fast tag lookups
        CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
        CREATE INDEX IF NOT EXISTS idx_link_tags_tag_id ON link_tags(tag_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id);

        -- Full-text search preparation (FTS5)
        -- Links: search title, url, description
        CREATE VIRTUAL TABLE IF NOT EXISTS links_fts USING fts5(
            title,
            url,
            description,
            content='links',
            content_rowid='rowid'
        );

        -- Notes: search title and body
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title,
            body,
            content='notes',
            content_rowid='rowid'
        );

        -- Triggers to keep FTS in sync with main tables

        -- Links FTS triggers
        CREATE TRIGGER IF NOT EXISTS links_ai AFTER INSERT ON links BEGIN
            INSERT INTO links_fts(rowid, title, url, description)
            VALUES (NEW.rowid, NEW.title, NEW.url, NEW.description);
        END;

        CREATE TRIGGER IF NOT EXISTS links_ad AFTER DELETE ON links BEGIN
            INSERT INTO links_fts(links_fts, rowid, title, url, description)
            VALUES ('delete', OLD.rowid, OLD.title, OLD.url, OLD.description);
        END;

        CREATE TRIGGER IF NOT EXISTS links_au AFTER UPDATE ON links BEGIN
            INSERT INTO links_fts(links_fts, rowid, title, url, description)
            VALUES ('delete', OLD.rowid, OLD.title, OLD.url, OLD.description);
            INSERT INTO links_fts(rowid, title, url, description)
            VALUES (NEW.rowid, NEW.title, NEW.url, NEW.description);
        END;

        -- Notes FTS triggers
        CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, title, body)
            VALUES (NEW.rowid, NEW.title, NEW.body);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, body)
            VALUES ('delete', OLD.rowid, OLD.title, OLD.body);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, body)
            VALUES ('delete', OLD.rowid, OLD.title, OLD.body);
            INSERT INTO notes_fts(rowid, title, body)
            VALUES (NEW.rowid, NEW.title, NEW.body);
        END;
        "#,
    )?;

    // Set schema version
    conn.execute(
        "INSERT OR REPLACE INTO schema_info (key, value) VALUES ('version', ?)",
        [SCHEMA_VERSION.to_string()],
    )?;

    Ok(())
}

/// Get the current schema version from the database
pub fn get_schema_version(conn: &Connection) -> Result<Option<i32>> {
    let mut stmt = conn.prepare("SELECT value FROM schema_info WHERE key = 'version'")?;
    let result: Result<String> = stmt.query_row([], |row| row.get(0));

    match result {
        Ok(version_str) => Ok(version_str.parse().ok()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Check if schema needs initialization or migration
pub fn needs_init(conn: &Connection) -> bool {
    // Check if schema_info table exists
    let table_exists: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_info'")
        .and_then(|mut stmt| stmt.exists([]))
        .unwrap_or(false);

    if !table_exists {
        return true;
    }

    match get_schema_version(conn) {
        Ok(Some(v)) => v < SCHEMA_VERSION,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_schema() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"links".to_string()));
        assert!(tables.contains(&"notes".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"link_tags".to_string()));
        assert!(tables.contains(&"note_tags".to_string()));
        assert!(tables.contains(&"link_authors".to_string()));
    }

    #[test]
    fn test_schema_version() {
        let conn = Connection::open_in_memory().unwrap();

        // Before init, needs init
        assert!(needs_init(&conn));

        init_schema(&conn).unwrap();

        // After init, has version and doesn't need init
        assert_eq!(get_schema_version(&conn).unwrap(), Some(SCHEMA_VERSION));
        assert!(!needs_init(&conn));
    }

    #[test]
    fn test_fts_tables_exist() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        // Verify FTS tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%_fts%'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.iter().any(|t| t.contains("links_fts")));
        assert!(tables.iter().any(|t| t.contains("notes_fts")));
    }

    #[test]
    fn test_indexes_exist() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_links_url".to_string()));
        assert!(indexes.contains(&"idx_links_created_at".to_string()));
        assert!(indexes.contains(&"idx_tags_name".to_string()));
    }
}
