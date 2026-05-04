//! Chat client for TorrentsTrackers.
//!
//! Connects to one or more community-hosted chat servers over WebSocket+TLS.
//! Authenticates via ed25519 signature. Caches messages in the local SQLite
//! storage. Phase 4.

pub mod client;
pub mod connection;
pub mod protocol;
