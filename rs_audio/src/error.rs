#[derive(Debug)]
pub enum Error {
    PlayStreamError(cpal::PlayStreamError),
    DevicesError(cpal::DevicesError),
    DeviceNameError(cpal::DeviceNameError),
    DefaultStreamConfigError(cpal::DefaultStreamConfigError),
    BuildStreamError(cpal::BuildStreamError),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
