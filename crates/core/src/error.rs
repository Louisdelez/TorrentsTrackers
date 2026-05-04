use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid signature")]
    InvalidSignature,

    #[error("invalid magnet link: {0}")]
    InvalidMagnet(String),

    #[error("source error: {0}")]
    Source(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
