use crate::{
    base_compute_pipeline::BaseComputePipeline,
    global_shaders::{global_shader::GlobalShader, panorama_to_cube::PanoramaToCubeShader},
    shader_library::ShaderLibrary,
};

pub struct PanoramaToCubePipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl PanoramaToCubePipeline {
    pub fn new(device: &wgpu::Device, shader_library: &ShaderLibrary) -> PanoramaToCubePipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &PanoramaToCubeShader {}.get_name());
        PanoramaToCubePipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        equirectangular_texture: &wgpu::Texture,
        length: u32,
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

        let cube_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("ibl_env_cube_map_texture")),
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PanoramaToCubeShader::get_format(),
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let cube_map_texture_view = cube_map_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("ibl_env__cube_map_texture_view")),
            format: Some(cube_map_texture.format()),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                wgpu::BindingResource::TextureView(&equirectangular_texture_view),
                wgpu::BindingResource::TextureView(&cube_map_texture_view),
            ]],
            glam::uvec3(length / 16, length / 16, 6),
        );
        cube_map_texture
    }
}
