#[derive(Debug)]
pub enum Error {
    RequestAdapterFailed,
    CreateSurfaceError(wgpu::CreateSurfaceError),
    RequestDeviceError(wgpu::RequestDeviceError),
    SurfaceNotSupported,
    Sync(Option<String>),
    ShaderReflection(naga::front::wgsl::ParseError, Option<String>),
    ShaderNotSupported(Option<String>),
    WindowError(raw_window_handle::HandleError),
    SurfaceError(wgpu::CreateSurfaceError),
    #[cfg(feature = "renderdoc")]
    RenderDoc(renderdoc::Error, Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
