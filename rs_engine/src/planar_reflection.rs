use crate::{build_built_in_resouce_url, engine::Engine, handle::TextureHandle};
use rs_render::command::{CreateTexture, RenderCommand, TextureDescriptorCreateInfo};

pub struct PlanarReflection {
    pub size: glam::UVec2,
    pub handle: TextureHandle,
}

impl PlanarReflection {
    pub fn new(engine: &mut Engine) -> crate::error::Result<PlanarReflection> {
        let size = glam::uvec2(1280, 720);
        let handle = engine.get_resource_manager().next_texture(
            build_built_in_resouce_url("PlanarReflectionRenderTarget")
                .map_err(|err| crate::error::Error::UrlParseError(err))?,
        );
        let command = RenderCommand::CreateTexture(CreateTexture {
            handle: *handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo {
                label: Some(format!("PlanarReflectionRenderTarget")),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: None,
            },
            init_data: None,
        });
        engine.send_render_command(command);
        Ok(PlanarReflection { size, handle })
    }
}
