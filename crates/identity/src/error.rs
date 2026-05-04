use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("ed25519 error: {0}")]
    Ed25519(String),

    #[error("bech32 error: {0}")]
    Bech32(String),

    #[error("keyring backend error: {0}")]
    Keyring(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("data dir not available")]
    NoDataDir,

    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid seed length (expected 32 bytes, got {0})")]
    InvalidSeedLength(usize),

    #[error("decryption failed (wrong passphrase or corrupted file)")]
    Decryption,

    #[error("invalid backup file format: {0}")]
    BackupFormat(String),

    #[error("identity not initialized; run `tt identity init`")]
    NotInitialized,

    #[error("signature verification failed")]
    InvalidSignature,
}

pub type Result<T> = std::result::Result<T, IdentityError>;
