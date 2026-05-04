use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("librqbit error: {0}")]
    Rqbit(String),

    #[error("data dir not available")]
    NoDataDir,

    #[error("torrent not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, DownloadError>;
