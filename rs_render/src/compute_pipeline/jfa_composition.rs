use crate::{
    base_compute_pipeline::BaseComputePipeline,
    constants::SDF2DConstants,
    global_shaders::{global_shader::GlobalShader, jfa_composition::JFACompositionShader},
    gpu_buffer,
    shader_library::ShaderLibrary,
};
use wgpu::*;

pub struct JFACompositionComputePipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl JFACompositionComputePipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
    ) -> JFACompositionComputePipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &JFACompositionShader {}.get_name());
        JFACompositionComputePipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        original_texture: &wgpu::Texture,
        input_texture: &wgpu::Texture,
        channel: i32,
        threshold: f32,
    ) -> wgpu::Texture {
        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("JFACompositionComputePipeline.input_texture_view")),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let original_texture_view = original_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!(
                "JFACompositionComputePipeline.original_texture_view"
            )),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let output_texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("JFACompositionComputePipeline.output_texture")),
            size: Extent3d {
                width: input_texture.width(),
                height: input_texture.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba16Float],
        });
        let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!(
                "JFACompositionComputePipeline.output_texture_view"
            )),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let constants = SDF2DConstants { channel, threshold };

        let uniform_buf = gpu_buffer::uniform::from(
            device,
            &constants,
            Some(&format!("JFACompositionComputePipeline.Constants")),
        );

        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                wgpu::BindingResource::TextureView(&original_texture_view),
                wgpu::BindingResource::TextureView(&input_texture_view),
                wgpu::BindingResource::TextureView(&output_texture_view),
                uniform_buf.as_entire_binding(),
            ]],
            glam::uvec3(input_texture.width() / 1, input_texture.height() / 1, 1),
        );
        output_texture
    }
}
