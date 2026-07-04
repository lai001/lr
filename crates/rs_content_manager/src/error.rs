use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("kernel errors or filesystem errors: {0}")]
    Notify(#[from] notify_debouncer_full::notify::Error),

    #[error("An error produced by recursively walking a directory: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Missing value: {0}")]
    MissingValue(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
