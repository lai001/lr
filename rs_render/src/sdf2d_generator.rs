use crate::{
    compute_pipeline::{
        jfa::{JFAComputePipeline, JFATextures},
        jfa_composition::JFACompositionComputePipeline,
        sdf2d_preprocess::Sdf2dPreprocessComputePipeline,
    },
    shader_library::ShaderLibrary,
};
use wgpu::{util::DeviceExt, *};

pub struct Sdf2dGenerator {
    cs_pipeline: Sdf2dPreprocessComputePipeline,
    jfa_compute_pipeline: JFAComputePipeline,
    jfa_composition_pipeline: JFACompositionComputePipeline,
}

impl Sdf2dGenerator {
    pub fn new(device: &wgpu::Device, shader_library: &ShaderLibrary) -> Sdf2dGenerator {
        let cs_pipeline = Sdf2dPreprocessComputePipeline::new(device, shader_library);
        let jfa_compute_pipeline = JFAComputePipeline::new(device, shader_library);
        let jfacomposition_compute_pipeline =
            JFACompositionComputePipeline::new(device, shader_library);

        Sdf2dGenerator {
            cs_pipeline,
            jfa_compute_pipeline,
            jfa_composition_pipeline: jfacomposition_compute_pipeline,
        }
    }

    pub fn run(
        &mut self,
        device: &Device,
        queue: &Queue,
        image: &image::RgbaImage,
        channel: i32,
        threshold: f32,
    ) -> wgpu::Texture {
        let input_texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Sdf2dGenerator.input_texture"),
                size: Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[TextureFormat::Rgba8Unorm],
            },
            util::TextureDataOrder::LayerMajor,
            image.as_raw(),
        );

        let output_texture = device.create_texture(&TextureDescriptor {
            label: Some("Sdf2dGenerator.output_texture"),
            size: Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[TextureFormat::Rgba16Float],
        });

        self.cs_pipeline.execute(
            device,
            queue,
            &input_texture,
            &output_texture,
            channel,
            threshold,
        );

        let mut jfa_textures = JFATextures::new(device, queue, &output_texture);
        let mut step = glam::uvec2(
            (input_texture.width() + 1) >> 1,
            (input_texture.height() + 1) >> 1,
        );
        loop {
            if step.x > 1 || step.y > 1 {
                self.jfa_compute_pipeline.execute(
                    device,
                    queue,
                    jfa_textures.get_textures()[0],
                    jfa_textures.get_textures()[1],
                    step.as_vec2(),
                );
                step = glam::uvec2((step.x + 1) >> 1, (step.y + 1) >> 1);
                jfa_textures.reverse();
            } else {
                break;
            }
        }

        self.jfa_composition_pipeline.execute(
            device,
            queue,
            &input_texture,
            jfa_textures.get_textures()[1],
            channel,
            threshold,
        )
    }
}
