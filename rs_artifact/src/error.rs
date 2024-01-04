#[derive(Debug)]
pub enum Error {
    File(Option<String>),
    IO(std::io::Error, Option<String>),
    CheckIdentificationFail,
    DataConvertFail,
    Bincode(bincode::Error, Option<String>),
    ResourceTypeNotMatch,
    NotFound(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
