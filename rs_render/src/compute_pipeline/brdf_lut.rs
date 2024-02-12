use crate::{
    base_compute_pipeline::BaseComputePipeline, global_shaders::brdf_lut::BrdfLutShader,
    gpu_buffer, shader_library::ShaderLibrary,
};

struct Constants {
    sample_count: u32,
}

pub struct BrdfLutPipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl BrdfLutPipeline {
    pub fn new(device: &wgpu::Device, shader_library: &ShaderLibrary) -> BrdfLutPipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &BrdfLutShader {});
        BrdfLutPipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        length: u32,
        sample_count: u32,
    ) -> wgpu::Texture {
        let lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("ibl_brdf_lut_texture")),
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: BrdfLutShader::get_format(),
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let lut_texture_view = lut_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("ibl_brdf_lut_texture_view")),
            format: Some(lut_texture.format()),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let constants = Constants { sample_count };

        let uniform_buf = gpu_buffer::uniform::from(device, &constants, None);
        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![
                vec![wgpu::BindingResource::TextureView(&lut_texture_view)],
                vec![uniform_buf.as_entire_binding()],
            ],
            glam::uvec3(length / 16, length / 16, 1),
        );

        lut_texture
    }
}
