#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    CreateProjectFailed,
    OpenProjectFailed(Option<String>),
    ExportFailed(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
