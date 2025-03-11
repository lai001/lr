use crate::{
    base_compute_pipeline::BaseComputePipeline,
    global_shaders::{box_culling::BoxCullingShader, global_shader::GlobalShader},
    shader_library::ShaderLibrary,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct AABB {
    pub min: glam::Vec3,
    _pad_0: i32,
    pub max: glam::Vec3,
    _pad_1: i32,
    pub transformation: glam::Mat4,
}

impl AABB {
    pub fn new(min: glam::Vec3, max: glam::Vec3, transformation: glam::Mat4) -> AABB {
        AABB {
            min,
            _pad_0: 0,
            max,
            _pad_1: 0,
            transformation,
        }
    }
}

pub struct BoxCullingPipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl BoxCullingPipeline {
    pub fn new(device: &wgpu::Device, shader_library: &ShaderLibrary) -> BoxCullingPipeline {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &BoxCullingShader {}.get_name());
        BoxCullingPipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        global_constants: &wgpu::Buffer,
        depth_texture_depth_2d: &wgpu::TextureView,
        boxes: &Vec<AABB>,
    ) -> crate::error::Result<Vec<u32>> {
        let mut results: Vec<u32> = vec![];
        results.resize(boxes.len(), 0);
        let contents = rs_foundation::cast_to_raw_buffer(boxes);
        let boxes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("boxes"),
            contents,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        });
        let buffer_size = size_of::<u32>() * boxes.len();
        let results_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("results"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });
        let submission_index = self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                global_constants.as_entire_binding(),
                wgpu::BindingResource::TextureView(&depth_texture_depth_2d),
                boxes_buffer.as_entire_binding(),
                results_buffer.as_entire_binding(),
            ]],
            glam::uvec3(boxes.len() as u32, 1, 1),
        );
        let buffer_slice = results_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
        receiver
            .recv()
            .map_err(|err| crate::error::Error::Sync(Some(err.to_string())))?
            .map_err(|err| crate::error::Error::Sync(Some(err.to_string())))?;
        let padded_buffer = buffer_slice.get_mapped_range();
        assert_eq!(padded_buffer.len() / size_of::<u8>(), buffer_size);
        let buffer = rs_foundation::cast_to_raw_type_buffer(padded_buffer.as_ptr(), buffer_size);
        results.copy_from_slice(buffer);
        drop(padded_buffer);
        results_buffer.unmap();
        Ok(results)
    }
}
