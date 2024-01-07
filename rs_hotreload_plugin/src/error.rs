#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    Libloading(libloading::Error, Option<String>),
    Symbol(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
