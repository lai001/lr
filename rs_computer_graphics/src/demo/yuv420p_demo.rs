use crate::{
    brigde_data::image2d_vertex::Image2DVertex, file_manager::FileManager,
    render_pipeline::yuv420p_pipeline::YUV420pPipeline, util, yuv420p_image::YUV420pImage,
};
use std::path::Path;
use wgpu::{Device, Queue, Texture, TextureView};

pub struct YUV420PDemo {
    pipeline: YUV420pPipeline,
    yuv_texture: (Texture, Texture, Texture),
}

impl YUV420PDemo {
    pub fn new(
        device: &Device,
        queue: &Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> YUV420PDemo {
        let pipeline = YUV420pPipeline::new(device, texture_format);

        let image = YUV420pImage::from_file(
            Path::new(&FileManager::default().get_resource_path("UVGrid.yuv420p")),
            &glam::uvec2(128, 128),
        )
        .unwrap();
        let yuv_texture = util::textures_from_yuv420p_image(device, queue, &image);

        YUV420PDemo {
            pipeline,
            yuv_texture,
        }
    }
    pub fn render(
        &self,
        vertex: Vec<Image2DVertex>,
        device: &Device,
        output_view: &TextureView,
        queue: &Queue,
    ) {
        self.pipeline.render(
            vertex,
            device,
            output_view,
            queue,
            &self.yuv_texture.0,
            &self.yuv_texture.1,
            &self.yuv_texture.2,
        );
    }
}
