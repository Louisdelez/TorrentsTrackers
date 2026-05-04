use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("websocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid url: {0}")]
    InvalidUrl(String),

    #[error("connection closed")]
    Closed,

    #[error("auth rejected: {0}")]
    AuthRejected(String),

    #[error("invalid signature on incoming message")]
    InvalidSignature,

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("not authenticated")]
    NotAuthenticated,

    #[error("send queue full")]
    Backpressure,
}

pub type Result<T> = std::result::Result<T, ChatError>;
