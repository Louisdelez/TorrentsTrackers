//! Core domain types and logic for TorrentsTrackers.
//!
//! This crate defines the canonical data model (`Entry`, `Source`, `Pool`,
//! `Community`) and the trait that source adapters must implement.

pub mod adapter;
pub mod community;
pub mod entry;
pub mod error;
pub mod filter;
pub mod ids;
pub mod magnet;
pub mod parse;
pub mod pool;
pub mod source;

pub use adapter::*;
pub use community::*;
pub use entry::*;
pub use error::*;
pub use ids::*;
pub use pool::*;
pub use source::*;
