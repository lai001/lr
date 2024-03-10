#[derive(Debug)]
pub enum AssimpError {
    Import(String),
}

impl std::fmt::Display for AssimpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for AssimpError {}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    Nul(std::ffi::NulError),
    Assimp(AssimpError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
