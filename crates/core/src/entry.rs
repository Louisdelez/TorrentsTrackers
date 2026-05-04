use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{ContentId, SourceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: ContentId,
    pub title: String,
    pub link: ContentLink,
    pub category: Category,
    #[serde(default)]
    pub tags: Vec<String>,
    pub quality: Option<Quality>,
    #[serde(default)]
    pub languages: Vec<Language>,
    pub size_bytes: Option<u64>,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub added_at: DateTime<Utc>,
    pub contributor_pubkey: PublicKeyBytes,
    pub source_id: SourceId,
    pub signature: SignatureBytes,
    pub description: Option<String>,
    pub poster_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentLink {
    Magnet(String),
    TorrentUrl(String),
    InfoHash([u8; 20]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Films,
    Series,
    Games,
    Music,
    Books,
    Software,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    P480,
    P720,
    P1080,
    P2160,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    FR,
    VOSTFR,
    EN,
    Multi,
    Other(String),
}

/// 32-byte ed25519 public key. Newtype around bytes so we can derive Serialize
/// without pulling ed25519 deps into every consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKeyBytes(pub [u8; 32]);

/// 64-byte ed25519 signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBytes(#[serde(with = "serde_bytes_64")] pub [u8; 64]);

mod serde_bytes_64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        v.try_into()
            .map_err(|_| serde::de::Error::custom("expected 64-byte signature"))
    }
}
