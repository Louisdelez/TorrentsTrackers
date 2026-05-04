use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("data dir not available")]
    NoDataDir,

    #[error("not found")]
    NotFound,

    #[error("invalid value: {0}")]
    Invalid(String),

    #[error("invalid signature")]
    InvalidSignature,

    #[error("contributor is banned in this source")]
    Banned,
}

pub type Result<T> = std::result::Result<T, StorageError>;
