#[derive(Debug)]
pub enum Error {
    File(Option<String>),
    IO(std::io::Error, Option<String>),
    Artifact(rs_artifact::error::Error, Option<String>),
    ArtifactReaderNotSet,
    RendererError(rs_render::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
