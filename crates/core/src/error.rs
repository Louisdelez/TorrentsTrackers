use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid signature")]
    InvalidSignature,

    #[error("invalid magnet link: {0}")]
    InvalidMagnet(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
