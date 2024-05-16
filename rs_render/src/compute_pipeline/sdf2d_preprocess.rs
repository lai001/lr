use crate::{
    base_compute_pipeline::BaseComputePipeline,
    global_shaders::sdf2d_preprocess::Sdf2dPreprocessShader, gpu_buffer,
    shader_library::ShaderLibrary,
};

struct Constants {
    channel: i32,
    threshold: f32,
}

pub struct Sdf2dPreprocessComputePipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl Sdf2dPreprocessComputePipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
    ) -> Sdf2dPreprocessComputePipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &Sdf2dPreprocessShader {});
        Sdf2dPreprocessComputePipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_texture: &wgpu::Texture,
        output_texture: &wgpu::Texture,
        channel: i32,
        threshold: f32,
    ) {
        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!(
                "Sdf2dPreprocessComputePipeline.input_texture_view"
            )),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!(
                "Sdf2dPreprocessComputePipeline.output_texture_view"
            )),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let constants = Constants { channel, threshold };

        let uniform_buf = gpu_buffer::uniform::from(
            device,
            &constants,
            Some(&format!("Sdf2dPreprocessComputePipeline.Constants")),
        );

        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                wgpu::BindingResource::TextureView(&input_texture_view),
                wgpu::BindingResource::TextureView(&output_texture_view),
                uniform_buf.as_entire_binding(),
            ]],
            glam::uvec3(input_texture.width() / 1, input_texture.height() / 1, 1),
        );
    }
}
