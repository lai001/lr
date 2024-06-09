use crate::build_built_in_resouce_url;
use crate::engine::{Engine, VirtualPassHandle};
use crate::handle::TextureHandle;
use glam::Vec4Swizzles;
use rs_render::antialias_type::FXAAInfo;
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, DrawObject, RenderCommand, TextureDescriptorCreateInfo,
};
use rs_render::global_uniform;
use rs_render::{antialias_type::EAntialiasType, scene_viewport::SceneViewport};

pub struct PlayerViewport {
    pub window_id: isize,
    pub scene_viewport: SceneViewport,
    pub width: u32,
    pub height: u32,
    pub global_sampler_handle: crate::handle::SamplerHandle,
    pub global_constants: rs_render::global_uniform::Constants,
    pub global_constants_handle: crate::handle::BufferHandle,
    pub virtual_pass_handle: Option<VirtualPassHandle>,
    pub shadow_depth_texture_handle: Option<TextureHandle>,
    pub grid_draw_object: Option<DrawObject>,
}

impl PlayerViewport {
    pub fn new(
        window_id: isize,
        width: u32,
        height: u32,
        global_sampler_handle: crate::handle::SamplerHandle,
        engine: &mut Engine,
    ) -> PlayerViewport {
        let scene_viewport = SceneViewport::new();

        let global_constants_handle = engine.get_resource_manager().next_buffer();
        let global_constants = global_uniform::Constants::default();
        let command = RenderCommand::CreateBuffer(CreateBuffer {
            handle: *global_constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some("Global.Constants".to_string()),
                contents: rs_foundation::cast_to_raw_buffer(&vec![global_constants]).to_vec(),
                usage: wgpu::BufferUsages::all(),
            },
        });
        engine.get_render_thread_mode_mut().send_command(command);

        PlayerViewport {
            scene_viewport,
            window_id,
            width,
            height,
            global_sampler_handle,
            virtual_pass_handle: None,
            shadow_depth_texture_handle: None,
            grid_draw_object: None,
            global_constants,
            global_constants_handle,
        }
    }

    pub fn enable_fxaa(&mut self, engine: &mut Engine) {
        let size = self
            .scene_viewport
            .viewport
            .as_ref()
            .map_or(glam::uvec2(self.width, self.height), |x| {
                x.rect.zw().floor().as_uvec2()
            });
        let texture_handle = engine.create_texture(
            &build_built_in_resouce_url("FXAATexture").unwrap(),
            TextureDescriptorCreateInfo {
                label: Some(format!("FXAATexture")),
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
        );

        self.scene_viewport.anti_type = EAntialiasType::FXAA(FXAAInfo {
            sampler: *self.global_sampler_handle,
            texture: *texture_handle,
        });
    }

    pub fn size_changed(&mut self, width: u32, height: u32, engine: &mut Engine) {
        self.width = width;
        self.height = height;
        match self.scene_viewport.anti_type {
            EAntialiasType::None => {}
            EAntialiasType::FXAA(_) => {
                self.enable_fxaa(engine);
            }
        }
    }

    // pub fn enable_shadow(&mut self, engine: &mut Engine) {
    //     let shadow_depth_texture_handle = engine
    //         .get_resource_manager()
    //         .next_texture(build_built_in_resouce_url("ShadowDepthTexture").unwrap());
    //     engine
    //         .get_render_thread_mode_mut()
    //         .send_command(RenderCommand::CreateTexture(CreateTexture {
    //             handle: *shadow_depth_texture_handle,
    //             texture_descriptor_create_info: TextureDescriptorCreateInfo {
    //                 label: Some(format!("ShadowDepthTexture")),
    //                 size: wgpu::Extent3d {
    //                     width: 1024,
    //                     height: 1024,
    //                     depth_or_array_layers: 1,
    //                 },
    //                 mip_level_count: 1,
    //                 sample_count: 1,
    //                 dimension: wgpu::TextureDimension::D2,
    //                 format: wgpu::TextureFormat::Depth32Float,
    //                 usage: wgpu::TextureUsages::RENDER_ATTACHMENT
    //                     | wgpu::TextureUsages::COPY_SRC
    //                     | wgpu::TextureUsages::TEXTURE_BINDING,
    //                 view_formats: None,
    //             },
    //             init_data: None,
    //         }));
    //     self.shadow_depth_texture_handle = Some(shadow_depth_texture_handle);
    // }

    // fn enable_virtual_texture(
    //     &mut self,
    //     engine: &mut Engine,
    //     virtual_texture_setting: VirtualTextureSetting,
    // ) {
    //     let handle = VirtualPassHandle::new();
    //     engine
    //         .get_render_thread_mode_mut()
    //         .send_command(RenderCommand::CreateVirtualTexturePass(
    //             CreateVirtualTexturePass {
    //                 key: handle.key(),
    //                 surface_size: glam::uvec2(self.width, self.height),
    //                 settings: virtual_texture_setting,
    //             },
    //         ));
    //     self.virtual_pass_handle = Some(handle);
    // }
}
