#[cfg(feature = "wgpu26")]
use wgpu26 as wgpu;

#[derive(Debug)]
pub enum Error {
    CreateSurfaceError(wgpu::CreateSurfaceError),
    RequestDeviceError(wgpu::RequestDeviceError),
    RequestAdapterError(wgpu::RequestAdapterError),
    SurfaceNotSupported,
    WindowError(raw_window_handle::HandleError),
    Sync(Option<String>),
    Other(Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
