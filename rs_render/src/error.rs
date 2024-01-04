#[derive(Debug)]
pub enum Error {
    RequestAdapterFailed,
    CreateSurfaceError(wgpu::CreateSurfaceError),
    RequestDeviceError(wgpu::RequestDeviceError),
    SurfaceNotSupported,
}

pub type Result<T> = std::result::Result<T, Error>;
