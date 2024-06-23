#[derive(Debug)]
pub enum Error {
    File(Option<String>),
    IO(std::io::Error, Option<String>),
    ImageError(image::ImageError, Option<String>),
    Artifact(rs_artifact::error::Error, Option<String>),
    ArtifactReaderNotSet,
    RendererError(rs_render::error::Error),
    RecvError(std::sync::mpsc::RecvError),
    NullReference(Option<String>),
    UrlParseError(url::ParseError),
    Other(Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
