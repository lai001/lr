use crate::{
    depth_texture::DepthTexture,
    frame_buffer::FrameBuffer,
    gpu_vertex_buffer::GpuVertexBufferImp,
    render_pipeline::{
        virtual_texture_feed_back::{Constants, VirtualTextureFeedBackPipeline},
        virtual_texture_feed_back_clean::VirtualTextureFeedBackClearPipeline,
    },
    shader_library::ShaderLibrary,
};
use rs_core_minimal::settings::VirtualTextureSetting;
use wgpu::*;

pub struct VirtualTexturePass {
    settings: VirtualTextureSetting,
    feeb_back_pipeline: VirtualTextureFeedBackPipeline,
    virtual_texture_feed_back_clear_pipeline: VirtualTextureFeedBackClearPipeline,
    feeb_back_frame_buffer: FrameBuffer,
    feed_back_size: glam::UVec2,
    feed_back_texture_format: TextureFormat,
}

impl VirtualTexturePass {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        is_noninterleaved: bool,
        surface_size: glam::UVec2,
        settings: VirtualTextureSetting,
    ) -> Self {
        let feed_back_size = surface_size / settings.feed_back_texture_div;
        let feed_back_texture_format = TextureFormat::Rgba32Uint;
        let feeb_back_frame_buffer =
            Self::create_feeb_back_frame_buffer(device, feed_back_texture_format, feed_back_size);
        let feeb_back_pipeline = VirtualTextureFeedBackPipeline::new(
            device,
            shader_library,
            &feed_back_texture_format,
            is_noninterleaved,
        );
        let virtual_texture_feed_back_clear_pipeline = VirtualTextureFeedBackClearPipeline::new(
            device,
            shader_library,
            &feed_back_texture_format,
        );

        Self {
            settings,
            feeb_back_pipeline,
            feeb_back_frame_buffer,
            feed_back_size,
            feed_back_texture_format,
            virtual_texture_feed_back_clear_pipeline,
        }
    }

    fn create_feeb_back_frame_buffer(
        device: &Device,
        texture_format: TextureFormat,
        feed_back_size: glam::UVec2,
    ) -> FrameBuffer {
        let depth_texture = DepthTexture::new(
            feed_back_size.x,
            feed_back_size.y,
            device,
            Some("VirtualTexturePass.DepthTexture"),
        );
        FrameBuffer::new(
            device,
            feed_back_size,
            texture_format,
            Some(depth_texture),
            Some("VirtualTexturePass.FeedbackFrameBuffer"),
        )
    }

    pub fn change_surface_size(&mut self, device: &Device, surface_size: glam::UVec2) {
        self.feed_back_size = surface_size / self.settings.feed_back_texture_div;
        self.feeb_back_frame_buffer = Self::create_feeb_back_frame_buffer(
            device,
            self.feed_back_texture_format,
            self.feed_back_size,
        );
    }

    pub fn begin_new_frame(&self, device: &Device, queue: &Queue) {
        self.virtual_texture_feed_back_clear_pipeline.draw(
            device,
            queue,
            &self.feeb_back_frame_buffer.get_color_texture_view(),
            &self
                .feeb_back_frame_buffer
                .get_depth_texture_view()
                .expect("Not null"),
        );
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        mesh_buffers: &[GpuVertexBufferImp],
        model: glam::Mat4,
        view: glam::Mat4,
        projection: glam::Mat4,
    ) {
        let constants = Constants {
            model,
            view,
            projection,
            physical_texture_size: self.settings.physical_texture_size,
            virtual_texture_size: self.settings.virtual_texture_size,
            tile_size: self.settings.tile_size,
            feed_back_texture_width: self.feed_back_size.x,
            feed_back_texture_height: self.feed_back_size.y,
            mipmap_level_bias: self.settings.mipmap_level_bias,
            mipmap_level_scale: self.settings.mipmap_level_scale,
            feedback_bias: self.settings.feedback_bias,
        };
        self.feeb_back_pipeline.draw(
            device,
            queue,
            &self.feeb_back_frame_buffer.get_color_texture_view(),
            &self
                .feeb_back_frame_buffer
                .get_depth_texture_view()
                .expect("Not null"),
            &constants,
            mesh_buffers,
        );
    }
}
