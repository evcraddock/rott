//! Storage layer
//!
//! Handles Automerge document persistence and SQLite projection.
//!
//! ## Architecture
//!
//! - **Automerge**: Source of truth, stored as binary file
//! - **SQLite**: Read-optimized projection for fast queries
//!
//! When the Automerge document changes, the SQLite database is updated
//! to reflect the new state.
//!
//! ## Error Handling
//!
//! The storage layer provides detailed error types for common issues:
//! - Disk full / quota exceeded
//! - Permission denied
//! - Corrupt documents (with automatic backup)
//! - Missing directories (auto-created)

pub mod error;
pub mod persistence;
pub mod projection;
pub mod schema;

pub use error::{StorageError, StorageResult};
pub use persistence::{AutomergePersistence, StorageStats};
pub use projection::SqliteProjection;
pub use schema::{init_schema, needs_init, SCHEMA_VERSION};
