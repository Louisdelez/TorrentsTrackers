use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoolId(pub Uuid);

impl SourceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SourceId {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PoolId {
    fn default() -> Self {
        Self::new()
    }
}
