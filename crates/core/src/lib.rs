//! Core domain types and logic for TorrentsTrackers.
//!
//! This crate defines the canonical data model (`Entry`, `Source`, `Pool`,
//! `Community`) and the trait that source adapters must implement.

pub mod entry;
pub mod source;
pub mod pool;
pub mod community;
pub mod ids;
pub mod adapter;
pub mod error;

pub use entry::*;
pub use source::*;
pub use pool::*;
pub use community::*;
pub use ids::*;
pub use adapter::*;
pub use error::*;
