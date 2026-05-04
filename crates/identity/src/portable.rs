//! Encrypted backup format for the 32-byte ed25519 seed.
//!
//! Layout (little-endian, all integers u32 unless noted):
//!
//! ```text
//! magic       "tt-id-v1\n"     (9 bytes)
//! salt_len    u32              (always 16)
//! salt        bytes
//! nonce_len   u32              (always 12)
//! nonce       bytes
//! n           u32              (scrypt log2_n parameter)
//! r           u32
//! p           u32
//! ct_len      u32
//! ciphertext  bytes  (= 32-byte seed encrypted with AES-256-GCM)
//! ```
//!
//! The ciphertext includes the 16-byte GCM tag, so its length is 48.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use rand::RngCore;
use rand::rngs::OsRng;
use scrypt::Params;
use zeroize::Zeroize;

use crate::error::{IdentityError, Result};

const MAGIC: &[u8] = b"tt-id-v1\n";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const SEED_LEN: usize = 32;

/// Default scrypt parameters: log2_n = 15 (~32 MB, ~100 ms on a desktop CPU).
const DEFAULT_LOG_N: u8 = 15;
const DEFAULT_R: u32 = 8;
const DEFAULT_P: u32 = 1;

pub fn export(seed: &[u8; SEED_LEN], passphrase: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let mut key = derive_key(passphrase, &salt, DEFAULT_LOG_N, DEFAULT_R, DEFAULT_P)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, seed.as_slice())
        .map_err(|_| IdentityError::Decryption)?;
    key.zeroize();

    let mut out =
        Vec::with_capacity(MAGIC.len() + 4 + SALT_LEN + 4 + NONCE_LEN + 16 + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(SALT_LEN as u32).to_le_bytes());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&(NONCE_LEN as u32).to_le_bytes());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&(DEFAULT_LOG_N as u32).to_le_bytes());
    out.extend_from_slice(&DEFAULT_R.to_le_bytes());
    out.extend_from_slice(&DEFAULT_P.to_le_bytes());
    out.extend_from_slice(&(ciphertext.len() as u32).to_le_bytes());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn import(blob: &[u8], passphrase: &str) -> Result<[u8; SEED_LEN]> {
    let mut cur = 0usize;

    fn read_slice<'a>(blob: &'a [u8], cur: &mut usize, n: usize) -> Result<&'a [u8]> {
        if *cur + n > blob.len() {
            return Err(IdentityError::BackupFormat("truncated input".to_string()));
        }
        let s = &blob[*cur..*cur + n];
        *cur += n;
        Ok(s)
    }

    fn read_u32(blob: &[u8], cur: &mut usize) -> Result<u32> {
        let s = read_slice(blob, cur, 4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }

    if read_slice(blob, &mut cur, MAGIC.len())? != MAGIC {
        return Err(IdentityError::BackupFormat("bad magic".to_string()));
    }
    let salt_len = read_u32(blob, &mut cur)? as usize;
    if salt_len != SALT_LEN {
        return Err(IdentityError::BackupFormat(format!(
            "salt_len = {salt_len}, expected {SALT_LEN}"
        )));
    }
    let salt = read_slice(blob, &mut cur, SALT_LEN)?.to_vec();
    let nonce_len = read_u32(blob, &mut cur)? as usize;
    if nonce_len != NONCE_LEN {
        return Err(IdentityError::BackupFormat(format!(
            "nonce_len = {nonce_len}, expected {NONCE_LEN}"
        )));
    }
    let nonce_bytes = read_slice(blob, &mut cur, NONCE_LEN)?.to_vec();
    let log_n = read_u32(blob, &mut cur)? as u8;
    let r = read_u32(blob, &mut cur)?;
    let p = read_u32(blob, &mut cur)?;
    let ct_len = read_u32(blob, &mut cur)? as usize;
    let ciphertext = read_slice(blob, &mut cur, ct_len)?.to_vec();

    let mut key = derive_key(passphrase, &salt, log_n, r, p)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|_| IdentityError::Decryption)?;
    key.zeroize();

    let seed: [u8; SEED_LEN] = plaintext
        .try_into()
        .map_err(|_: Vec<u8>| IdentityError::BackupFormat("seed length wrong".to_string()))?;
    Ok(seed)
}

fn derive_key(passphrase: &str, salt: &[u8], log_n: u8, r: u32, p: u32) -> Result<[u8; 32]> {
    let params = Params::new(log_n, r, p, 32)
        .map_err(|e| IdentityError::BackupFormat(format!("scrypt params: {e}")))?;
    let mut out = [0u8; 32];
    scrypt::scrypt(passphrase.as_bytes(), salt, &params, &mut out)
        .map_err(|e| IdentityError::BackupFormat(format!("scrypt: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_import_roundtrip() {
        let seed = [42u8; 32];
        let blob = export(&seed, "correct horse battery staple").unwrap();
        let recovered = import(&blob, "correct horse battery staple").unwrap();
        assert_eq!(seed, recovered);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let seed = [42u8; 32];
        let blob = export(&seed, "right").unwrap();
        let res = import(&blob, "wrong");
        assert!(matches!(res, Err(IdentityError::Decryption)));
    }

    #[test]
    fn corrupted_blob_fails() {
        let seed = [42u8; 32];
        let mut blob = export(&seed, "pass").unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xff;
        assert!(import(&blob, "pass").is_err());
    }
}
