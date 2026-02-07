//! Storage layer
//!
//! Handles Automerge document persistence.
//!
//! ## Architecture
//!
//! - **Automerge**: Source of truth, stored as binary file
//!
//! All queries are served directly from the in-memory Automerge document.
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

pub use error::{StorageError, StorageResult};
pub use persistence::{AutomergePersistence, StorageStats};
