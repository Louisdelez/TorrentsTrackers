//! SQLite storage layer for TorrentsTrackers.
//!
//! Owns the local database (entries, sources, pools, messages, bans) and
//! provides typed read/write APIs to the rest of the workspace.

pub mod db;
pub mod error;
pub mod migrations;
pub mod paths;
pub mod queries;

pub use db::Database;
pub use error::{Result, StorageError};
pub use queries::{LocalIdentity, SearchHit, SearchQuery, SearchScope};
