use crate::{
    shader::shader_library::ShaderLibrary,
    virtual_texture::{packing::ArrayTile, tile_index::TileIndex},
};
use std::collections::HashMap;
use wgpu::*;

#[repr(C)]
#[derive(Debug)]
struct Element {
    virtual_index_x: i32,
    virtual_index_y: i32,
    physical_offset_x: i32,
    physical_offset_y: i32,
    physical_array_index: i32,
    virtual_mimap: i32,
}

pub struct UpdatePageTableCSPipeline {
    compute_pipeline: ComputePipeline,
    textures_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
}

impl UpdatePageTableCSPipeline {
    pub fn new(device: &wgpu::Device) -> UpdatePageTableCSPipeline {
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("update_page_table.cs.wgsl");
        let textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UpdatePageTableCSPipeline.textures_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Uint,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UpdatePageTableCSPipeline.uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Element>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&textures_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });
        UpdatePageTableCSPipeline {
            compute_pipeline,
            textures_bind_group_layout,
            uniform_bind_group_layout,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        page_table: &wgpu::Texture,
        pack_result: &HashMap<TileIndex, ArrayTile>,
    ) {
        debug_assert_eq!(page_table.format(), TextureFormat::Rgba16Uint);
        let page_table_texture_view =
            page_table.create_view(&wgpu::TextureViewDescriptor::default());

        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.textures_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&page_table_texture_view),
            }],
        });

        let mut elements: Vec<Element> = Vec::new();
        for (k, v) in pack_result {
            let element = Element {
                physical_array_index: v.index as i32,
                virtual_mimap: k.mipmap_level as i32,
                virtual_index_x: k.tile_offset.x as i32,
                virtual_index_y: k.tile_offset.y as i32,
                physical_offset_x: v.offset_x as i32,
                physical_offset_y: v.offset_y as i32,
            };
            elements.push(element);
        }

        let uniform_buf =
            crate::util::create_gpu_uniform_buffer_from_array(device, &elements, None);

        let constants_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &textures_bind_group, &[]);
            cpass.set_bind_group(1, &constants_bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }
        let _ = queue.submit(Some(encoder.finish()));
    }
}
