use crate::{
    base_compute_pipeline::BaseComputePipeline,
    constants::{ClusterLightIndex, Sphere3D},
    global_shaders::{global_shader::GlobalShader, light_culling::LightCullingShader},
    global_uniform::CameraFrustum,
    shader_library::ShaderLibrary,
};
use wgpu::{util::DeviceExt, BufferUsages};

pub struct ExecuteResult {
    pub point_light_shapes: wgpu::Buffer,
    pub frustums: wgpu::Buffer,
    pub cluster_lights: wgpu::Buffer,
    pub cluster_light_indices: wgpu::Buffer,
}

pub struct LightCullingComputePipeline {
    base_compute_pipeline: BaseComputePipeline,
}

impl LightCullingComputePipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
    ) -> crate::error::Result<LightCullingComputePipeline> {
        let base_compute_pipeline =
            BaseComputePipeline::new(device, shader_library, &LightCullingShader {}.get_name());
        Ok(LightCullingComputePipeline {
            base_compute_pipeline,
        })
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        point_light_shapes: &wgpu::Buffer,
        frustum: &wgpu::Buffer,
        cluster_lights: &wgpu::Buffer,
        cluster_light_indices: &wgpu::Buffer,
        workgroups: glam::UVec3,
    ) {
        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                point_light_shapes.as_entire_binding(),
                frustum.as_entire_binding(),
                cluster_lights.as_entire_binding(),
                cluster_light_indices.as_entire_binding(),
            ]],
            workgroups,
        );
    }

    pub fn execute_out(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        point_light_shapes: &Vec<Sphere3D>,
        frustum: &CameraFrustum,
        cluster_lights: &Vec<u32>,
        cluster_light_indices: &Vec<ClusterLightIndex>,
        workgroups: glam::UVec3,
    ) -> ExecuteResult {
        let point_light_shapes = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: rs_foundation::cast_to_raw_buffer(point_light_shapes),
            usage: BufferUsages::STORAGE,
        });
        let frustums = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: rs_foundation::cast_any_as_u8_slice(frustum),
            usage: BufferUsages::UNIFORM,
        });
        let cluster_lights = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: rs_foundation::cast_to_raw_buffer(cluster_lights),
            usage: BufferUsages::STORAGE,
        });
        let cluster_light_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: rs_foundation::cast_to_raw_buffer(cluster_light_indices),
            usage: BufferUsages::STORAGE,
        });

        self.execute(
            device,
            queue,
            &point_light_shapes,
            &frustums,
            &cluster_lights,
            &cluster_light_indices,
            workgroups,
        );

        ExecuteResult {
            point_light_shapes,
            frustums,
            cluster_lights,
            cluster_light_indices,
        }
    }
}
