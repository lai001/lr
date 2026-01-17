#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    PatternError(glob::PatternError, Option<String>),
    GlobError(glob::GlobError, Option<String>),
    FromUtf8Error(std::string::FromUtf8Error),
    Utf8Error(std::str::Utf8Error),
    DdsfileError(image_dds::ddsfile::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
