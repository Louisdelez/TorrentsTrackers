//! Canonical payload for entry signatures.
//!
//! `signing_payload` produces a deterministic byte representation of an
//! [`Entry`] excluding its signature. Both signers and verifiers must derive
//! the bytes from this function so that signatures roundtrip across hosts.
//!
//! Format (all integers little-endian):
//!
//! ```text
//! magic        "tt-entry-v1\0"        (12 bytes)
//! id           [u8; 32]               (32 bytes)
//! title        u32 len | utf8 bytes
//! link_kind    u32 len | utf8 bytes
//! link_value   u32 len | utf8 bytes   (magnet, url, or 40-char hex info_hash)
//! category     u32 len | utf8 bytes
//! tags         u32 count | (u32 len | utf8)*
//! quality      u8 present | (u32 len | utf8)?    (canonical label)
//! languages    u32 count | (u32 len | utf8)*    (canonical labels)
//! size_bytes   u8 present | u64?
//! seeders      u8 present | u32?
//! leechers     u8 present | u32?
//! added_at     u32 len | rfc3339 utf8
//! contrib_pk   [u8; 32]
//! source_id    [u8; 16]               (uuid bytes)
//! description  u32 len | utf8 (empty if None)
//! poster_url   u32 len | utf8 (empty if None)
//! ```

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::entry::{Category, ContentLink, Entry, Language, PublicKeyBytes, Quality};
use crate::error::{CoreError, Result};

const MAGIC: &[u8] = b"tt-entry-v1\0";

pub fn signing_payload(entry: &Entry) -> Vec<u8> {
    let mut out = Vec::with_capacity(256);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&entry.id.0);

    push_str(&mut out, &entry.title);
    let (kind, value) = link_kv(&entry.link);
    push_str(&mut out, kind);
    push_str(&mut out, &value);
    push_str(&mut out, category_label(entry.category));

    push_u32(&mut out, entry.tags.len() as u32);
    for t in &entry.tags {
        push_str(&mut out, t);
    }

    match &entry.quality {
        Some(q) => {
            out.push(1);
            push_str(&mut out, &quality_label(q));
        }
        None => out.push(0),
    }

    push_u32(&mut out, entry.languages.len() as u32);
    for l in &entry.languages {
        push_str(&mut out, &language_label(l));
    }

    push_opt_u64(&mut out, entry.size_bytes);
    push_opt_u32(&mut out, entry.seeders);
    push_opt_u32(&mut out, entry.leechers);
    push_str(&mut out, &entry.added_at.to_rfc3339());
    out.extend_from_slice(&entry.contributor_pubkey.0);
    out.extend_from_slice(entry.source_id.0.as_bytes());
    push_str(&mut out, entry.description.as_deref().unwrap_or(""));
    push_str(&mut out, entry.poster_url.as_deref().unwrap_or(""));
    out
}

fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    push_u32(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
}

fn push_opt_u64(out: &mut Vec<u8>, v: Option<u64>) {
    match v {
        Some(x) => {
            out.push(1);
            out.extend_from_slice(&x.to_le_bytes());
        }
        None => out.push(0),
    }
}

fn push_opt_u32(out: &mut Vec<u8>, v: Option<u32>) {
    match v {
        Some(x) => {
            out.push(1);
            out.extend_from_slice(&x.to_le_bytes());
        }
        None => out.push(0),
    }
}

fn link_kv(link: &ContentLink) -> (&'static str, String) {
    match link {
        ContentLink::Magnet(s) => ("Magnet", s.clone()),
        ContentLink::TorrentUrl(s) => ("TorrentUrl", s.clone()),
        ContentLink::InfoHash(b) => ("InfoHash", hex::encode(b)),
    }
}

fn category_label(c: Category) -> &'static str {
    match c {
        Category::Films => "Films",
        Category::Series => "Series",
        Category::Games => "Games",
        Category::Music => "Music",
        Category::Books => "Books",
        Category::Software => "Software",
        Category::Other => "Other",
    }
}

fn quality_label(q: &Quality) -> String {
    match q {
        Quality::P480 => "480p".into(),
        Quality::P720 => "720p".into(),
        Quality::P1080 => "1080p".into(),
        Quality::P2160 => "2160p".into(),
        Quality::Other(s) => s.clone(),
    }
}

fn language_label(l: &Language) -> String {
    match l {
        Language::FR => "FR".into(),
        Language::VOSTFR => "VOSTFR".into(),
        Language::EN => "EN".into(),
        Language::Multi => "Multi".into(),
        Language::Other(s) => s.clone(),
    }
}

/// Verify the entry's signature against its declared `contributor_pubkey`.
/// Returns `Err(CoreError::InvalidSignature)` on failure.
pub fn verify_entry(entry: &Entry) -> Result<()> {
    verify_with(entry, &entry.contributor_pubkey)
}

/// Verify the signature against an explicit public key. Useful when the
/// caller wants to enforce that the signature comes from a specific
/// contributor.
pub fn verify_with(entry: &Entry, pk: &PublicKeyBytes) -> Result<()> {
    let vk = VerifyingKey::from_bytes(&pk.0).map_err(|_| CoreError::InvalidSignature)?;
    let sig = Signature::from_bytes(&entry.signature.0);
    let payload = signing_payload(entry);
    vk.verify(&payload, &sig)
        .map_err(|_| CoreError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{PublicKeyBytes, SignatureBytes};
    use crate::ids::{ContentId, SourceId};
    use chrono::TimeZone;

    fn sample() -> Entry {
        let link = ContentLink::Magnet(
            "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567".into(),
        );
        Entry {
            id: ContentId::compute(&link, "Inception 1080p").unwrap(),
            title: "Inception 1080p".into(),
            link,
            category: Category::Films,
            tags: vec!["1080p".into(), "vostfr".into()],
            quality: Some(Quality::P1080),
            languages: vec![Language::VOSTFR],
            size_bytes: Some(5_368_709_120),
            seeders: Some(234),
            leechers: Some(12),
            added_at: chrono::Utc
                .with_ymd_and_hms(2026, 4, 12, 18, 30, 0)
                .unwrap(),
            contributor_pubkey: PublicKeyBytes([7; 32]),
            source_id: SourceId(uuid::Uuid::nil()),
            signature: SignatureBytes([0; 64]),
            description: None,
            poster_url: None,
        }
    }

    #[test]
    fn payload_is_deterministic() {
        let a = signing_payload(&sample());
        let b = signing_payload(&sample());
        assert_eq!(a, b);
    }

    #[test]
    fn payload_changes_with_title() {
        let mut e = sample();
        let a = signing_payload(&e);
        e.title = "Inception 4K".into();
        let b = signing_payload(&e);
        assert_ne!(a, b);
    }

    #[test]
    fn payload_changes_with_pubkey() {
        let mut e = sample();
        let a = signing_payload(&e);
        e.contributor_pubkey.0[0] ^= 1;
        let b = signing_payload(&e);
        assert_ne!(a, b);
    }

    #[test]
    fn payload_ignores_signature_field() {
        let mut e = sample();
        let a = signing_payload(&e);
        e.signature.0[0] ^= 1;
        let b = signing_payload(&e);
        assert_eq!(a, b);
    }
}
