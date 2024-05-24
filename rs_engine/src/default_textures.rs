use crate::{
    build_built_in_resouce_url, handle::TextureHandle, render_thread_mode::ERenderThreadMode,
    resource_manager::ResourceManager,
};
use rs_render::command::{CreateTexture, RenderCommand, TextureDescriptorCreateInfo};
use wgpu::*;

pub struct DefaultTextures {
    texture_handle: TextureHandle,
    texture_cube_handle: TextureHandle,
}

impl DefaultTextures {
    pub fn new(rm: ResourceManager) -> DefaultTextures {
        DefaultTextures {
            texture_handle: rm.next_texture(build_built_in_resouce_url("DefaultTexture0").unwrap()),
            texture_cube_handle: rm
                .next_texture(build_built_in_resouce_url("DefaultCubeTexture0").unwrap()),
        }
    }

    pub fn create(&self, render_thread_mode: &mut ERenderThreadMode) {
        render_thread_mode.send_command(RenderCommand::CreateTexture(CreateTexture {
            handle: *self.texture_handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo {
                label: Some(format!("DefaultTexture0")),
                size: Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::all(),
                view_formats: None,
            },
            init_data: None,
        }));

        render_thread_mode.send_command(RenderCommand::CreateTexture(CreateTexture {
            handle: *self.texture_cube_handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo {
                label: Some(format!("DefaultCubeTexture0")),
                size: Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 6,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::all(),
                view_formats: None,
            },
            init_data: None,
        }));
    }

    pub fn get_texture_handle(&self) -> TextureHandle {
        self.texture_handle.clone()
    }

    pub fn get_texture_cube_handle(&self) -> TextureHandle {
        self.texture_cube_handle.clone()
    }
}
