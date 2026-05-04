//! Cryptographic identity for TorrentsTrackers.
//!
//! Generates and manages an ed25519 keypair stored in the OS keyring (or
//! encrypted on disk as a fallback). Signs and verifies entries.

pub mod keypair;
pub mod npub;
pub mod signing;
pub mod storage;
