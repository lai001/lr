use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    EndOfFile,
    TryAgain,
    Disconnected,
    Cpal(cpal::Error),
    IO(std::io::Error),
    MP4(mp4::Error),
    FromUtf8Error(FromUtf8Error),
    FFMpeg(ffmpeg_next::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
