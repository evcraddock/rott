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

pub mod persistence;
pub mod schema;

pub use persistence::AutomergePersistence;
pub use schema::{init_schema, needs_init, SCHEMA_VERSION};
