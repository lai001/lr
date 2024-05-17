use crate::{
    base_compute_pipeline::BaseComputePipeline, constants::IBLConstants,
    global_shaders::irradiance_cube_map::IrradianceCubeMapShader, gpu_buffer,
    shader_library::ShaderLibrary,
};

pub struct IrradianceCubeMapPipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl IrradianceCubeMapPipeline {
    pub fn new(device: &wgpu::Device, shader_library: &ShaderLibrary) -> IrradianceCubeMapPipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &IrradianceCubeMapShader {});
        IrradianceCubeMapPipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        equirectangular_texture: &wgpu::Texture,
        length: u32,
        sample_count: u32,
    ) -> wgpu::Texture {
        let equirectangular_texture_view =
            equirectangular_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(equirectangular_texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        let irradiance_cube_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("ibl_irradiance_cube_map_texture")),
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: IrradianceCubeMapShader::get_format(),
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let irradiance_cube_map_texture_view =
            irradiance_cube_map_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("ibl_irradiance_cube_map_texture_view")),
                format: Some(irradiance_cube_map_texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        let constants = IBLConstants { sample_count };
        let uniform_buf = gpu_buffer::uniform::from(device, &constants, None);

        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![
                vec![
                    wgpu::BindingResource::TextureView(&equirectangular_texture_view),
                    wgpu::BindingResource::TextureView(&irradiance_cube_map_texture_view),
                ],
                vec![uniform_buf.as_entire_binding()],
            ],
            glam::uvec3(length / 16, length / 16, 6),
        );
        irradiance_cube_map_texture
    }
}
