//! Magnet URI parsing and helpers.
//!
//! Format: `magnet:?xt=urn:btih:<hash>[&dn=<name>][&tr=<tracker>]...`
//! The btih hash may be 40-char hex (SHA-1 / BTv1) or 32-char base32.

use crate::error::{CoreError, Result};

/// Extract the BTv1 `info_hash` (20 bytes) from a magnet link.
///
/// Returns the hash as a lowercase 40-char hex string.
pub fn extract_info_hash(magnet: &str) -> Result<String> {
    if !magnet.starts_with("magnet:?") {
        return Err(CoreError::InvalidMagnet(
            "missing magnet:? prefix".to_string(),
        ));
    }

    let query = &magnet["magnet:?".len()..];
    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if k != "xt" {
            continue;
        }
        if let Some(rest) = v.strip_prefix("urn:btih:") {
            return parse_btih(rest);
        }
    }

    Err(CoreError::InvalidMagnet(
        "no xt=urn:btih: found".to_string(),
    ))
}

fn parse_btih(s: &str) -> Result<String> {
    if s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(s.to_ascii_lowercase());
    }
    if s.len() == 32 {
        // Base32 encoded — decode to bytes then re-encode to hex.
        let bytes = base32_decode(s)
            .ok_or_else(|| CoreError::InvalidMagnet("invalid base32 btih".to_string()))?;
        if bytes.len() != 20 {
            return Err(CoreError::InvalidMagnet(format!(
                "btih base32 decoded to {} bytes, expected 20",
                bytes.len()
            )));
        }
        return Ok(hex::encode(bytes));
    }
    Err(CoreError::InvalidMagnet(format!(
        "btih has unexpected length {}",
        s.len()
    )))
}

/// Decode a 32-char RFC 4648 base32 string (uppercase, no padding) to bytes.
fn base32_decode(s: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut bits: u32 = 0;
    let mut nbits: u32 = 0;
    let mut out = Vec::with_capacity(s.len() * 5 / 8);
    for c in s.bytes() {
        let v = ALPHABET.iter().position(|&a| a == c.to_ascii_uppercase())?;
        bits = (bits << 5) | (v as u32);
        nbits += 5;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
            bits &= (1 << nbits) - 1;
        }
    }
    Some(out)
}

/// Build a minimal magnet URI from a 20-byte info_hash and an optional name.
pub fn build_magnet(info_hash: &[u8; 20], display_name: Option<&str>) -> String {
    let mut s = format!("magnet:?xt=urn:btih:{}", hex::encode(info_hash));
    if let Some(name) = display_name {
        let encoded = url::form_urlencoded::byte_serialize(name.as_bytes()).collect::<String>();
        s.push_str("&dn=");
        s.push_str(&encoded);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_btih() {
        let m = "magnet:?xt=urn:btih:0123456789abcdef0123456789ABCDEF01234567&dn=test";
        let h = extract_info_hash(m).unwrap();
        assert_eq!(h, "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn parse_base32_btih() {
        // 20 bytes of 0x00 in base32 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        let m = "magnet:?xt=urn:btih:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let h = extract_info_hash(m).unwrap();
        assert_eq!(h, "0000000000000000000000000000000000000000");
    }

    #[test]
    fn rejects_non_magnet() {
        assert!(extract_info_hash("https://example.com/").is_err());
    }

    #[test]
    fn rejects_missing_xt() {
        assert!(extract_info_hash("magnet:?dn=foo").is_err());
    }

    #[test]
    fn build_magnet_roundtrip() {
        let hash = [0x12; 20];
        let m = build_magnet(&hash, Some("My File"));
        assert!(m.starts_with("magnet:?xt=urn:btih:"));
        assert!(m.contains("dn=My+File") || m.contains("dn=My%20File"));
        let parsed = extract_info_hash(&m).unwrap();
        assert_eq!(parsed, hex::encode(hash));
    }
}
