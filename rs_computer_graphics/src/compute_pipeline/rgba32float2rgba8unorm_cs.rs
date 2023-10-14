use super::base_compute_pipeline::BaseComputePipeline;
use wgpu::*;

pub struct Rgba32float2rgba8unormCSPipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl Rgba32float2rgba8unormCSPipeline {
    pub fn new(device: &Device) -> Rgba32float2rgba8unormCSPipeline {
        let base_compute_pipeline = BaseComputePipeline::new(device, "rgba32float2rgba8unorm.wgsl");

        Rgba32float2rgba8unormCSPipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(&self, device: &Device, queue: &Queue, source_texture: &Texture) -> Texture {
        debug_assert_eq!(source_texture.format(), TextureFormat::Rgba32Float);
        debug_assert_eq!(source_texture.dimension(), TextureDimension::D2);
        debug_assert!(source_texture
            .usage()
            .contains(TextureUsages::STORAGE_BINDING));

        let target_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: source_texture.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                BindingResource::TextureView(
                    &source_texture.create_view(&TextureViewDescriptor::default()),
                ),
                BindingResource::TextureView(
                    &target_texture.create_view(&TextureViewDescriptor::default()),
                ),
            ]],
            glam::uvec3(1, 1, 1),
        );

        target_texture
    }
}
