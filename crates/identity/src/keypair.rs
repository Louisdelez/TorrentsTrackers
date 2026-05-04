//! ed25519 keypair, generation and accessors.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use tt_core::PublicKeyBytes;
use zeroize::Zeroize;

use crate::error::{IdentityError, Result};
use crate::npub;

/// A locally-held ed25519 keypair.
///
/// The 32-byte seed is held in `signing` and zeroized when the value is
/// dropped (via [`zeroize::ZeroizeOnDrop`] from `ed25519-dalek`).
pub struct LocalKeypair {
    pub(crate) signing: SigningKey,
}

impl LocalKeypair {
    /// Generate a fresh keypair using the OS RNG.
    pub fn generate() -> Self {
        let signing = SigningKey::generate(&mut OsRng);
        Self { signing }
    }

    /// Reconstruct a keypair from a 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(seed);
        Self { signing }
    }

    pub fn from_seed_slice(seed: &[u8]) -> Result<Self> {
        let arr: [u8; 32] = seed
            .try_into()
            .map_err(|_| IdentityError::InvalidSeedLength(seed.len()))?;
        Ok(Self::from_seed(&arr))
    }

    /// Return the 32-byte seed (private). Caller is responsible for zeroizing.
    pub fn seed(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing.verifying_key()
    }

    pub fn public_bytes(&self) -> PublicKeyBytes {
        PublicKeyBytes(self.verifying_key().to_bytes())
    }

    /// `npub1...` bech32 representation of the public key.
    pub fn npub(&self) -> String {
        npub::encode_npub(&self.public_bytes()).expect("npub encode never fails for valid pubkey")
    }

    /// `nsec1...` bech32 representation of the seed (handle with care).
    pub fn nsec(&self) -> String {
        let seed = self.seed();
        let s = npub::encode_nsec(&seed).expect("nsec encode never fails");
        let mut z = seed;
        z.zeroize();
        s
    }

    /// Raw sign of an arbitrary byte slice. Prefer [`crate::signing::sign_entry`]
    /// for entries.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing.sign(msg)
    }
}

impl std::fmt::Debug for LocalKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalKeypair")
            .field("npub", &self.npub())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_roundtrip_seed() {
        let kp = LocalKeypair::generate();
        let seed = kp.seed();
        let kp2 = LocalKeypair::from_seed(&seed);
        assert_eq!(kp.public_bytes().0, kp2.public_bytes().0);
    }

    #[test]
    fn npub_starts_with_prefix() {
        let kp = LocalKeypair::generate();
        let n = kp.npub();
        assert!(n.starts_with("npub1"), "npub: {n}");
    }

    #[test]
    fn nsec_starts_with_prefix() {
        let kp = LocalKeypair::generate();
        let n = kp.nsec();
        assert!(n.starts_with("nsec1"), "nsec: {n}");
    }
}
