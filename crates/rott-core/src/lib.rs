//! ROTT Core Library
//!
//! This crate provides the core functionality for ROTT (Read Over The Top),
//! a local-first knowledge management system for links and notes.
//!
//! # Architecture
//!
//! - **Automerge**: Source of truth for data, enables CRDT-based sync
//!
//! All queries are served directly from the in-memory Automerge document.
//!
//! # Quick Start
//!
//! ```text
//! let mut store = Store::open()?;
//!
//! // Add a link
//! let mut link = Link::new("https://example.com");
//! link.set_title("Example");
//! store.add_link(&link)?;
//!
//! // Query links
//! let links = store.get_all_links()?;
//! ```
//!
//! # Modules
//!
//! - `store`: Unified storage interface (main entry point)
//! - `models`: Data structures for links, notes, and tags
//! - `document`: Automerge document handling
//! - `document_id`: Document ID compatible with automerge-repo
//! - `storage`: Automerge persistence
//! - `config`: Application configuration

pub mod config;
pub mod document;
pub mod document_id;
pub mod identity;
pub mod models;
pub mod storage;
pub mod store;
pub mod sync;

pub use config::Config;
pub use document::{DocumentError, RottDocument};
pub use document_id::{DocumentId, DocumentIdError};
pub use identity::{Identity, InitResult};
pub use models::{Link, Note, Tag};
pub use storage::{AutomergePersistence, StorageError, StorageStats};
pub use store::Store;
