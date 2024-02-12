#[derive(Debug)]
pub enum Error {
    RequestAdapterFailed,
    CreateSurfaceError(wgpu::CreateSurfaceError),
    RequestDeviceError(wgpu::RequestDeviceError),
    SurfaceNotSupported,
    Sync(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
