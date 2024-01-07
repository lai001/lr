#[derive(Debug)]
pub enum Error {
    IO(std::io::Error, Option<String>),
    CreateProjectFailed,
    OpenProjectFailed
}

pub type Result<T> = std::result::Result<T, Error>;
