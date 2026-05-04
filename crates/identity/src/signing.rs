//! Sign [`tt_core::Entry`] with the local keypair.
//!
//! Verification lives in [`tt_core::signing::verify_entry`] (no private key
//! is needed for that path). This module re-exports it for convenience.

use ed25519_dalek::Signature;
use tt_core::{Entry, SignatureBytes, signing::signing_payload};

pub use tt_core::signing::{verify_entry, verify_with};

use crate::keypair::LocalKeypair;

/// Sign an entry in place. Sets `contributor_pubkey`, `signature`.
/// The caller is responsible for setting `source_id`, `id`, `added_at`, etc.
/// before calling this.
pub fn sign_entry(entry: &mut Entry, key: &LocalKeypair) {
    entry.contributor_pubkey = key.public_bytes();
    let payload = signing_payload(entry);
    let sig: Signature = key.sign(&payload);
    entry.signature = SignatureBytes(sig.to_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tt_core::{
        Category, ContentId, ContentLink, Entry, Language, PublicKeyBytes, Quality, SourceId,
    };

    fn fixture(kp: &LocalKeypair) -> Entry {
        let link = ContentLink::Magnet(
            "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567".into(),
        );
        let mut e = Entry {
            id: ContentId::compute(&link, "Inception 1080p").unwrap(),
            title: "Inception 1080p".into(),
            link,
            category: Category::Films,
            tags: vec!["1080p".into()],
            quality: Some(Quality::P1080),
            languages: vec![Language::VOSTFR],
            size_bytes: Some(5_000_000_000),
            seeders: Some(50),
            leechers: Some(2),
            added_at: Utc::now(),
            contributor_pubkey: PublicKeyBytes([0; 32]),
            source_id: SourceId::new(),
            signature: SignatureBytes([0; 64]),
            description: None,
            poster_url: None,
        };
        sign_entry(&mut e, kp);
        e
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = LocalKeypair::generate();
        let e = fixture(&kp);
        verify_entry(&e).expect("valid signature");
    }

    #[test]
    fn tampering_breaks_signature() {
        let kp = LocalKeypair::generate();
        let mut e = fixture(&kp);
        e.title = "Inception 4K".into();
        assert!(verify_entry(&e).is_err());
    }

    #[test]
    fn wrong_pubkey_fails() {
        let kp = LocalKeypair::generate();
        let mut e = fixture(&kp);
        e.contributor_pubkey = LocalKeypair::generate().public_bytes();
        assert!(verify_entry(&e).is_err());
    }
}
