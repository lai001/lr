use crate::{
    base_compute_pipeline::BaseComputePipeline,
    constants::JFAConstants,
    global_shaders::{global_shader::GlobalShader, jfa::JFAShader},
    gpu_buffer,
    shader_library::ShaderLibrary,
};
use wgpu::*;

pub struct JFATextures {
    front: wgpu::Texture,
    back: wgpu::Texture,
    is_reverse: bool,
}

impl JFATextures {
    fn create_textuere(device: &wgpu::Device, width: u32, height: u32, name: &str) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some(&format!("Sdf2dGenerator.{name}")),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[TextureFormat::Rgba16Float],
        })
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> JFATextures {
        let width = texture.width();
        let height = texture.height();
        let front = Self::create_textuere(device, width, height, "front");
        let back = Self::create_textuere(device, width, height, "back");

        queue.submit([{
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Copy texture"),
            });
            encoder.copy_texture_to_texture(
                texture.as_image_copy(),
                front.as_image_copy(),
                front.size(),
            );
            encoder.copy_texture_to_texture(
                texture.as_image_copy(),
                back.as_image_copy(),
                back.size(),
            );
            encoder.finish()
        }]);
        JFATextures {
            front,
            back,
            is_reverse: false,
        }
    }

    pub fn get_textures(&self) -> Vec<&wgpu::Texture> {
        if self.is_reverse {
            vec![&self.back, &self.front]
        } else {
            vec![&self.front, &self.back]
        }
    }

    pub fn reverse(&mut self) {
        self.is_reverse = !self.is_reverse;
    }
}

pub struct JFAComputePipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl JFAComputePipeline {
    pub fn new(device: &wgpu::Device, shader_library: &ShaderLibrary) -> JFAComputePipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &JFAShader {}.get_name());
        JFAComputePipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_texture: &wgpu::Texture,
        output_texture: &wgpu::Texture,
        step: glam::Vec2,
    ) {
        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("JFAComputePipeline.input_texture_view")),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("JFAComputePipeline.output_texture_view")),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let constants = JFAConstants { step };

        let uniform_buf = gpu_buffer::uniform::from(
            device,
            &constants,
            Some(&format!("JFAComputePipeline.Constants")),
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
