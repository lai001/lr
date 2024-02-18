#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    Libloading(libloading::Error, Option<String>),
    Symbol(Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
