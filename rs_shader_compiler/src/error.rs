#[derive(Debug)]
pub enum Error {
    ProcessFail(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
