//! SQLite storage layer for TorrentsTrackers.
//!
//! Owns the local database (entries, sources, pools, messages, bans) and
//! provides typed read/write APIs to the rest of the workspace.

pub mod db;
pub mod migrations;
pub mod queries;
