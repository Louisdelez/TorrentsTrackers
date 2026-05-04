//! Source adapter implementations.
//!
//! Each module implements the `tt_core::SourceAdapter` trait for a specific
//! backend (local folder, HTTP URL, Git repo, cloud drive, Nostr relay, ...).

pub mod local;
pub mod http;

// Phase 2 adapters (stubs for now)
pub mod git;
pub mod nostr;
pub mod drive;
pub mod server;
