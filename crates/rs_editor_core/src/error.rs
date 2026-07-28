use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Missing value: {0}")]
    MissingValue(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
