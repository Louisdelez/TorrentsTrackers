//! Tauri IPC commands. Each module is a thin shim over the workspace crates,
//! converting Rust domain types to JSON-friendly DTOs the Svelte frontend
//! consumes.

pub mod chat;
pub mod dto;
pub mod identity;
pub mod magnet;
pub mod pool;
pub mod publish;
pub mod search;
pub mod source;
pub mod stats;
