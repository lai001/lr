#[derive(Debug)]
pub enum Error {
    File(Option<String>),
    IO(std::io::Error, Option<String>),
    RecvError(std::sync::mpsc::RecvError),
    Execute(String),
    Null(String),
    Debouncer(notify::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
