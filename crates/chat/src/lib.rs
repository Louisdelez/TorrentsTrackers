//! Chat client for TorrentsTrackers.
//!
//! Connects to one or more community-hosted chat servers over WebSocket
//! (with optional TLS via `wss://`). Authenticates via an ed25519 challenge-
//! response. Wire messages are JSON-serialized [`ServerMessage`] /
//! [`ClientMessage`] envelopes.

pub mod client;
pub mod connection;
pub mod error;
pub mod protocol;

pub use client::{ChatClient, ChatEvent};
pub use error::{ChatError, Result};
pub use protocol::{ChatMessage, ClientMessage, ServerMessage};
