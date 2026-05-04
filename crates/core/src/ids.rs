use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entry::ContentLink;
use crate::error::Result;
use crate::magnet::extract_info_hash;
use crate::parse::normalize_title;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(pub [u8; 32]);

impl ContentId {
    /// Compute a stable content id from a `ContentLink` and a normalized title.
    ///
    /// Two entries with the same info_hash and the same normalized title hash
    /// to the same `ContentId`, making cross-source deduplication possible
    /// even when the surrounding metadata (trackers, surrounding text) differ.
    pub fn compute(link: &ContentLink, title: &str) -> Result<Self> {
        let mut h = blake3::Hasher::new();
        match link {
            ContentLink::Magnet(m) => {
                let info_hash = extract_info_hash(m)?;
                h.update(b"btih:");
                h.update(info_hash.as_bytes());
            }
            ContentLink::TorrentUrl(u) => {
                h.update(b"url:");
                h.update(u.as_bytes());
            }
            ContentLink::InfoHash(bytes) => {
                h.update(b"btih:");
                h.update(bytes);
            }
        }
        h.update(b"\x00");
        h.update(normalize_title(title).as_bytes());
        Ok(ContentId(h.finalize().into()))
    }

    pub fn as_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> std::result::Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(ContentId(arr))
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_hex())
    }
}

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

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

impl std::fmt::Display for PoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
