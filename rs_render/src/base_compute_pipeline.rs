use crate::{
    global_shaders::global_shader::GlobalShader, reflection::EPipelineType,
    shader_library::ShaderLibrary,
};
use wgpu::*;

pub struct BaseComputePipeline {
    compute_pipeline: ComputePipeline,
    bind_group_layouts: Vec<BindGroupLayout>,
    tag: String,
}

impl BaseComputePipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
        global_shader: &impl GlobalShader,
    ) -> BaseComputePipeline {
        let tag = &global_shader.get_name();

        let shader = shader_library.get_shader(tag);
        let reflection = shader_library.get_shader_reflection(tag);
        let EPipelineType::Compute(cs) = reflection.get_pipeline_type() else {
            panic!()
        };
        let bind_group_layout_entrys = reflection.get_bind_group_layout_entrys();

        let mut bind_group_layouts: Vec<BindGroupLayout> = Vec::new();
        for bind_group_layout_entry_vec in bind_group_layout_entrys {
            let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(&format!("{} bind group layout", tag)),
                entries: bind_group_layout_entry_vec,
            });
            bind_group_layouts.push(bind_group_layout);
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} pipeline layout", tag)),
            bind_group_layouts: &bind_group_layouts
                .iter()
                .map(|x| x)
                .collect::<Vec<&BindGroupLayout>>(),
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("{} compute pipeline", tag)),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: &cs.name,
        });
        BaseComputePipeline {
            compute_pipeline,
            bind_group_layouts,
            tag: tag.clone(),
        }
    }

    pub fn execute_entries(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        entries: Vec<Vec<BindGroupEntry>>,
        workgroups: glam::UVec3,
    ) -> SubmissionIndex {
        let mut bind_groups: Vec<BindGroup> = Vec::new();
        for (entry_vec, bind_group_layout) in entries.iter().zip(self.bind_group_layouts.iter()) {
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &entry_vec,
                label: Some(&format!("{} bind group", self.tag)),
            });
            bind_groups.push(bind_group);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{} command encoder", self.tag)),
        });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!("{} compute pass", self.tag)),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            for (index, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(index as u32, bind_group, &[]);
            }
            compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        }
        queue.submit(Some(encoder.finish()))
    }

    pub fn execute_resources(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        binding_resources: Vec<Vec<BindingResource>>,
        workgroups: glam::UVec3,
    ) -> SubmissionIndex {
        let entries = binding_resources
            .iter()
            .map(|x| {
                x.iter()
                    .enumerate()
                    .map(|(binding, resource)| wgpu::BindGroupEntry {
                        binding: binding as u32,
                        resource: resource.clone(),
                    })
                    .collect()
            })
            .collect();
        self.execute_entries(device, queue, entries, workgroups)
    }
}
