//! Cryptographic identity for TorrentsTrackers.
//!
//! Generates and manages an ed25519 keypair stored in the OS keyring (or
//! plain-file fallback under $XDG_CONFIG_HOME). Signs and verifies entries,
//! exports/imports the seed via AES-GCM with an scrypt-derived key.

pub mod error;
pub mod keypair;
pub mod npub;
pub mod portable;
pub mod signing;
pub mod storage;

pub use error::{IdentityError, Result};
pub use keypair::LocalKeypair;
pub use signing::{sign_entry, verify_entry};
pub use storage::{DefaultStore, FileStore, IdentityStore, KeyringStore};
