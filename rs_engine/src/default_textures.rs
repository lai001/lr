use crate::{
    build_built_in_resouce_url,
    handle::TextureHandle,
    render_thread_mode::ERenderThreadMode,
    resource_manager::{IBLTextures, ResourceManager},
};
use rs_render::{
    command::{CreateTexture, RenderCommand, TextureDescriptorCreateInfo},
    misc::find_most_compatible_texture_usages,
};
use wgpu::*;

pub struct DefaultTextures {
    texture_handle: TextureHandle,
    texture_cube_handle: TextureHandle,
    ibl_textures: IBLTextures,
    depth_texture_handle: TextureHandle,
}

impl DefaultTextures {
    pub fn new(rm: ResourceManager) -> DefaultTextures {
        DefaultTextures {
            texture_handle: rm.next_texture(build_built_in_resouce_url("DefaultTexture0").unwrap()),
            texture_cube_handle: rm
                .next_texture(build_built_in_resouce_url("DefaultCubeTexture0").unwrap()),
            ibl_textures: rm.next_ibl_textures(build_built_in_resouce_url("IBLTextures0").unwrap()),
            depth_texture_handle: rm
                .next_texture(build_built_in_resouce_url("ShadowDepthTexture").unwrap()),
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
                usage: find_most_compatible_texture_usages(TextureFormat::Rgba8Unorm),
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
                usage: find_most_compatible_texture_usages(TextureFormat::Rgba8Unorm),
                view_formats: None,
            },
            init_data: None,
        }));

        render_thread_mode
            .send_command(RenderCommand::CreateDefaultIBL(self.ibl_textures.to_key()));

        render_thread_mode.send_command(RenderCommand::CreateTexture(CreateTexture {
            handle: *self.depth_texture_handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo {
                label: Some(format!("ShadowDepthTexture")),
                size: wgpu::Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
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

    pub fn get_ibl_textures(&self) -> &IBLTextures {
        &self.ibl_textures
    }

    pub fn get_depth_texture_handle(&self) -> TextureHandle {
        self.depth_texture_handle.clone()
    }
}
