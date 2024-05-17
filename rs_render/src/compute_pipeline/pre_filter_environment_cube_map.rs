use crate::{
    base_compute_pipeline::BaseComputePipeline, constants::PreFilterConstants,
    global_shaders::pre_filter_environment_cube_map::PreFilterEnvironmentCubeMapShader, gpu_buffer,
    shader_library::ShaderLibrary,
};

pub struct PreFilterEnvironmentCubeMapComputePipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl PreFilterEnvironmentCubeMapComputePipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
    ) -> PreFilterEnvironmentCubeMapComputePipeline {
        let base_compute_pipeline = BaseComputePipeline::new(
            device,
            shader_library,
            &PreFilterEnvironmentCubeMapShader {},
        );

        PreFilterEnvironmentCubeMapComputePipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        equirectangular_texture: &wgpu::Texture,
        length: u32,
        roughness: f32,
        sample_count: u32,
    ) -> wgpu::Texture {
        let equirectangular_texture_view =
            equirectangular_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("equirectangular_texture_view")),
                format: Some(equirectangular_texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        let prefilter_cube_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("ibl_prefilter_cube_map_texture_{:.2}", roughness)),
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PreFilterEnvironmentCubeMapShader::get_format(),
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let prefilter_cube_map_texture_view =
            prefilter_cube_map_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!(
                    "ibl_prefilter_cube_map_texture_view_{:.2}",
                    roughness
                )),
                format: Some(prefilter_cube_map_texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        let constants = PreFilterConstants {
            roughness,
            sample_count,
        };

        let uniform_buf = gpu_buffer::uniform::from(device, &constants, None);

        let size = (length / 16).max(1);
        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![
                vec![
                    wgpu::BindingResource::TextureView(&equirectangular_texture_view),
                    wgpu::BindingResource::TextureView(&prefilter_cube_map_texture_view),
                ],
                vec![uniform_buf.as_entire_binding()],
            ],
            glam::uvec3(size, size, 6),
        );
        return prefilter_cube_map_texture;
    }
}
