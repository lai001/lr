use std::sync::Mutex;

#[derive(Debug)]
pub enum Error {
    Sync(Option<String>),
    ShaderReflection(naga::front::wgsl::ParseError, Option<String>),
    ShaderNotSupported(Option<String>),
    WindowError(raw_window_handle::HandleError),
    SurfaceError(wgpu::CreateSurfaceError),
    ImageError(image::error::ImageError),
    #[cfg(feature = "renderdoc")]
    RenderDoc(renderdoc::Error, Option<String>),
    ImageDdsSurface(image_dds::error::SurfaceError),
    ImageDdsCreateImage(image_dds::error::CreateImageError),
    ImageDdsCreateDds(image_dds::CreateDdsError),
    DdsFile(ddsfile::Error),
    IO(std::io::Error, Option<String>),
    Wgpu(Mutex<wgpu::Error>),
    ValidationError(naga::WithSpan<naga::valid::ValidationError>),
    NagaBackSpirVError(naga::back::spv::Error),
    RenderCore(rs_render_core::error::Error),
    Other(Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
