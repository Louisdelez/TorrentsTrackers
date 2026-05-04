//! Source adapter implementations.
//!
//! Each module implements the `tt_core::SourceAdapter` trait for a specific
//! backend (local folder, HTTP URL, Git repo, cloud drive, Nostr relay, ...).

pub mod http;
pub mod local;

// Phase 2 adapters (stubs for now)
pub mod drive;
pub mod git;
pub mod nostr;
pub mod server;
