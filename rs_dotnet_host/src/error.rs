#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    Debouncer(notify::Error),
    Dotnet(rs_dotnet::error::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
