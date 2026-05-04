//! `npub1...` / `nsec1...` bech32 encoding (compatible with Nostr NIP-19 raw keys).

use bech32::{Bech32, Hrp};
use tt_core::PublicKeyBytes;

use crate::error::{IdentityError, Result};

const HRP_NPUB: &str = "npub";
const HRP_NSEC: &str = "nsec";

pub fn encode_npub(pk: &PublicKeyBytes) -> Result<String> {
    let hrp = Hrp::parse(HRP_NPUB).expect("static hrp");
    bech32::encode::<Bech32>(hrp, &pk.0).map_err(|e| IdentityError::Bech32(e.to_string()))
}

pub fn encode_nsec(seed: &[u8; 32]) -> Result<String> {
    let hrp = Hrp::parse(HRP_NSEC).expect("static hrp");
    bech32::encode::<Bech32>(hrp, seed).map_err(|e| IdentityError::Bech32(e.to_string()))
}

pub fn decode_npub(s: &str) -> Result<PublicKeyBytes> {
    let (hrp, data) = bech32::decode(s).map_err(|e| IdentityError::Bech32(e.to_string()))?;
    if hrp.as_str() != HRP_NPUB {
        return Err(IdentityError::Bech32(format!(
            "expected hrp 'npub', got '{}'",
            hrp.as_str()
        )));
    }
    let bytes: [u8; 32] = data
        .try_into()
        .map_err(|v: Vec<u8>| IdentityError::Bech32(format!("bad pubkey length {}", v.len())))?;
    Ok(PublicKeyBytes(bytes))
}

pub fn decode_nsec(s: &str) -> Result<[u8; 32]> {
    let (hrp, data) = bech32::decode(s).map_err(|e| IdentityError::Bech32(e.to_string()))?;
    if hrp.as_str() != HRP_NSEC {
        return Err(IdentityError::Bech32(format!(
            "expected hrp 'nsec', got '{}'",
            hrp.as_str()
        )));
    }
    data.try_into()
        .map_err(|v: Vec<u8>| IdentityError::Bech32(format!("bad seed length {}", v.len())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npub_roundtrip() {
        let pk = PublicKeyBytes([42; 32]);
        let s = encode_npub(&pk).unwrap();
        let pk2 = decode_npub(&s).unwrap();
        assert_eq!(pk.0, pk2.0);
    }

    #[test]
    fn nsec_roundtrip() {
        let seed = [9u8; 32];
        let s = encode_nsec(&seed).unwrap();
        let seed2 = decode_nsec(&s).unwrap();
        assert_eq!(seed, seed2);
    }

    #[test]
    fn rejects_wrong_hrp() {
        let pk = PublicKeyBytes([42; 32]);
        let s = encode_npub(&pk).unwrap();
        assert!(decode_nsec(&s).is_err());
    }
}
