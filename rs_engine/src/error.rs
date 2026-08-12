use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("File error: {0:?}")]
    File(Option<String>),
    #[error("{1:?}: {0}")]
    IO(std::io::Error, Option<String>),
    #[error("{1:?}: {0}")]
    ImageError(image::ImageError, Option<String>),
    #[error(transparent)]
    Artifact(#[from] rs_artifact::error::Error),
    #[error("Artifact reader is not set")]
    ArtifactReaderNotSet,
    #[error(transparent)]
    RendererError(#[from] rs_render::error::Error),
    #[error(transparent)]
    RecvError(#[from] std::sync::mpsc::RecvError),
    #[error("Null reference: {0:?}")]
    NullReference(Option<String>),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    AudioError(#[from] rs_audio::error::Error),
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error(
        "Downcast failed: expected {expected}, but the boxed asset was of a different type (url: {url})"
    )]
    DowncastFail {
        expected: &'static str,
        url: url::Url,
    },
    #[error("Buffer is too small")]
    BufferTooSmall,
    #[error("{0:?}")]
    Other(Option<String>),
}

impl From<Box<dyn rs_artifact_types::asset::Asset>> for Error {
    fn from(boxed: Box<dyn rs_artifact_types::asset::Asset>) -> Self {
        Error::DowncastFail {
            expected: std::any::type_name::<dyn rs_artifact_types::asset::Asset>(),
            url: boxed.get_url(),
        }
    }
}

impl From<Box<dyn rs_content::Content>> for Error {
    fn from(boxed: Box<dyn rs_content::Content>) -> Self {
        Error::DowncastFail {
            expected: std::any::type_name::<dyn rs_content::Content>(),
            url: boxed.get_url(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
