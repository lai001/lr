use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Compilation error: {0}")]
    Compilation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
