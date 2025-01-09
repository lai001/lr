#[derive(Debug)]
pub enum Error {
    ProcessFail(Option<String>),
    IO(std::io::Error, Option<String>),
    FromUtf8Error(std::string::FromUtf8Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
