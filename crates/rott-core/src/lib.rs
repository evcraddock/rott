//! ROTT Core Library
//!
//! This crate provides the core functionality for ROTT (Read Over The Top),
//! a local-first knowledge management system for links and notes.
//!
//! # Architecture
//!
//! - **Automerge**: Source of truth for data, enables CRDT-based sync
//! - **SQLite**: Read-optimized projection for fast queries
//!
//! # Modules
//!
//! - `models`: Data structures for links, notes, and tags
//! - `document`: Automerge document handling
//! - `document_id`: Document ID compatible with automerge-repo
//! - `storage`: Automerge persistence and SQLite projection
//! - `config`: Application configuration

pub mod config;
pub mod document;
pub mod document_id;
pub mod models;
pub mod storage;

pub use config::Config;
pub use document::{DocumentError, RottDocument};
pub use document_id::{DocumentId, DocumentIdError};
pub use models::{Link, Note, Tag};
pub use storage::{AutomergePersistence, SqliteProjection};
