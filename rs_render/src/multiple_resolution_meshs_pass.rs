use crate::{
    base_render_pipeline_pool::BaseRenderPipelinePool,
    compute_pipeline::box_culling::{BoxCullingPipeline, AABB},
    global_shaders::{global_shader::GlobalShader, view_depth::ViewDepthShader},
    gpu_vertex_buffer::{Draw, EDrawCallType, GpuVertexBufferImp},
    misc::find_or_insert_bind_groups,
    multi_res_mesh::MultipleResolutionMesh,
    render_pipeline::generic_pipeline::GenericPipeline,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::MeshVertex5,
    VertexBufferType,
};
use std::{collections::HashMap, iter::zip, ops::Range, sync::Arc};
use type_layout::TypeLayout;
use wgpu::util::DeviceExt;

pub struct ClusterResource {
    pub id: i32,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct ClusterCollectionResource {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub cluster_resources: HashMap<i32, ClusterResource>,
}

pub struct MultipleResolutionMeshsPass {
    pub cluster_collection_resources: HashMap<u64, ClusterCollectionResource>,
    pub depth_pipeline: GenericPipeline,
    pub box_culling_pipeline: BoxCullingPipeline,
}

impl MultipleResolutionMeshsPass {
    pub fn new(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
    ) -> MultipleResolutionMeshsPass {
        let cluster_collection_resources = HashMap::new();
        let depth_pipeline = GenericPipeline::standard_depth_only(
            ViewDepthShader {}.get_name(),
            device,
            shader_library,
            pool,
            Some(VertexBufferType::Interleaved(vec![
                MeshVertex5::type_layout(),
            ])),
        );

        let box_culling_pipeline = BoxCullingPipeline::new(device, shader_library);

        MultipleResolutionMeshsPass {
            cluster_collection_resources,
            depth_pipeline,
            box_culling_pipeline,
        }
    }

    pub fn create_resource(
        &mut self,
        device: &wgpu::Device,
        id: u64,
        multiple_resolution_mesh: &MultipleResolutionMesh,
    ) {
        let cluster_collection_resource = Self::_create_resource(device, multiple_resolution_mesh);
        self.cluster_collection_resources
            .insert(id, cluster_collection_resource);
    }

    fn _create_resource(
        device: &wgpu::Device,
        multiple_resolution_mesh: &MultipleResolutionMesh,
    ) -> ClusterCollectionResource {
        let mut vertex_buffer: Vec<MeshVertex5> =
            Vec::with_capacity(multiple_resolution_mesh.vertexes.len());
        for item in &multiple_resolution_mesh.vertexes {
            vertex_buffer.push(MeshVertex5 {
                position: item.position,
            });
        }
        let occluder_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: rs_foundation::cast_to_raw_buffer(&vertex_buffer),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut cluster_resources = HashMap::new();

        for (cluster_id, cluster) in &multiple_resolution_mesh.cluster_collection.clusters {
            let occluder_index_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: rs_foundation::cast_to_raw_buffer(&cluster.occluder_indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
            cluster_resources.insert(
                *cluster_id,
                ClusterResource {
                    id: *cluster_id,
                    index_buffer: occluder_index_buffer,
                    index_count: cluster.occluder_indices.len() as u32,
                },
            );
        }

        ClusterCollectionResource {
            vertex_buffer: occluder_vertex_buffer,
            cluster_resources,
            vertex_count: vertex_buffer.len() as u32,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        multiple_resolution_meshes: &HashMap<u64, MultipleResolutionMesh>,
        texture_views: HashMap<u64, &wgpu::TextureView>,
        buffers: &HashMap<u64, Arc<wgpu::Buffer>>,
        samplers: &HashMap<u64, wgpu::Sampler>,
        draw_objects: Vec<&crate::command::DrawObject>,
        depth_view: &wgpu::TextureView,
        bind_groups_collection: &mut moka::sync::Cache<u64, Arc<Vec<wgpu::BindGroup>>>,
    ) -> wgpu::SubmissionIndex {
        let render_pipeline = self.depth_pipeline.base_render_pipeline.clone();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!(
                "{} command encoder",
                "MultipleResolutionMeshsPass"
            )),
        });
        {
            let depth_stencil_attachment: wgpu::RenderPassDepthStencilAttachment =
                wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("{} render pass", "MultipleResolutionMeshsPass")),
                color_attachments: &vec![],
                depth_stencil_attachment: Some(depth_stencil_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for draw_object in draw_objects {
                let Some(multiple_resolution_mesh_pass) =
                    &draw_object.multiple_resolution_mesh_pass
                else {
                    continue;
                };
                let Some(cluster_collection_resource) = self
                    .cluster_collection_resources
                    .get(&multiple_resolution_mesh_pass.resource_handle)
                else {
                    continue;
                };
                let Some(multiple_resolution_mesh) =
                    multiple_resolution_meshes.get(&multiple_resolution_mesh_pass.resource_handle)
                else {
                    continue;
                };
                let Some(root_cluster_resource) = cluster_collection_resource
                    .cluster_resources
                    .get(&multiple_resolution_mesh.cluster_collection.root_id)
                else {
                    continue;
                };

                let gpu_vertex_buffer = GpuVertexBufferImp {
                    vertex_buffers: &vec![&cluster_collection_resource.vertex_buffer],
                    vertex_count: cluster_collection_resource.vertex_count,
                    index_buffer: Some(&root_cluster_resource.index_buffer),
                    index_count: Some(root_cluster_resource.index_count),
                    draw_type: EDrawCallType::Draw(Draw { instances: 0..1 }),
                };
                let bind_groups = find_or_insert_bind_groups(
                    device,
                    &render_pipeline,
                    &texture_views,
                    buffers,
                    samplers,
                    &multiple_resolution_mesh_pass.binding_resources,
                    bind_groups_collection,
                )
                .expect("Find bind groups");
                render_pipeline.draw_with_pass(
                    &mut render_pass,
                    &bind_groups,
                    &vec![gpu_vertex_buffer],
                    None,
                    None,
                    None,
                );
            }
        }
        queue.submit(Some(encoder.finish()))
    }

    pub fn instance_culling(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffers: &HashMap<u64, Arc<wgpu::Buffer>>,
        draw_objects: Vec<&crate::command::DrawObject>,
        depth_view: &wgpu::TextureView,
        multiple_resolution_meshes: &HashMap<u64, MultipleResolutionMesh>,
        global_constant_resources: crate::command::EBindingResource,
    ) -> Option<Vec<usize>> {
        let crate::command::EBindingResource::Constants(global_constant_handle) =
            global_constant_resources
        else {
            return None;
        };
        let Some(global_constants_buffer) = buffers.get(&global_constant_handle) else {
            return None;
        };
        let mut boxes: Vec<AABB> = Vec::with_capacity(draw_objects.len());

        for draw_object in draw_objects {
            let Some(multiple_resolution_mesh_pass) = &draw_object.multiple_resolution_mesh_pass
            else {
                return None;
            };
            let Some(multiple_resolution_mesh) =
                multiple_resolution_meshes.get(&multiple_resolution_mesh_pass.resource_handle)
            else {
                return None;
            };
            let Some(root_aabb) = multiple_resolution_mesh.cluster_collection.get_root_aabb()
            else {
                return None;
            };
            let aabb = AABB::new(
                type_convertion(root_aabb.mins),
                type_convertion(root_aabb.maxs),
                multiple_resolution_mesh_pass.transformation,
            );
            boxes.push(aabb);
        }

        let culling_results = self
            .box_culling_pipeline
            .execute(device, queue, global_constants_buffer, depth_view, &boxes)
            .ok()?;

        assert_eq!(culling_results.len(), boxes.len());

        Some(
            culling_results
                .iter()
                .enumerate()
                .filter(|(_, result)| **result == 1)
                .map(|(i, _)| i)
                .collect(),
        )
    }

    pub fn cluster_culling(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffers: &HashMap<u64, Arc<wgpu::Buffer>>,
        draw_objects: Vec<&crate::command::DrawObject>,
        depth_view: &wgpu::TextureView,
        multiple_resolution_meshes: &HashMap<u64, MultipleResolutionMesh>,
        global_constant_resources: crate::command::EBindingResource,
    ) -> Option<Vec<Vec<i32>>> {
        let crate::command::EBindingResource::Constants(global_constant_handle) =
            global_constant_resources
        else {
            return None;
        };
        let Some(global_constants_buffer) = buffers.get(&global_constant_handle) else {
            return None;
        };
        let mut boxes: Vec<AABB> = vec![];
        let mut ranges: Vec<Range<usize>> = Vec::with_capacity(draw_objects.len());
        let mut cluster_ids: Vec<i32> = vec![];
        let mut start: usize = 0;

        let mut visible_cluster_ids: Vec<Vec<i32>> = Vec::with_capacity(draw_objects.len());

        for draw_object in draw_objects.iter() {
            let Some(multiple_resolution_mesh_pass) = &draw_object.multiple_resolution_mesh_pass
            else {
                return None;
            };
            let Some(multiple_resolution_mesh) =
                multiple_resolution_meshes.get(&multiple_resolution_mesh_pass.resource_handle)
            else {
                return None;
            };

            let mut leaf_cluster_ids = multiple_resolution_mesh
                .cluster_collection
                .get_leaf_cluster_ids();

            let end = start + leaf_cluster_ids.len();
            ranges.push(Range { start, end });
            start = end;

            cluster_ids.append(&mut leaf_cluster_ids);

            let mut aabbs: Vec<AABB> = multiple_resolution_mesh
                .cluster_collection
                .get_leaf_clusters()
                .iter()
                .map(|x| {
                    AABB::new(
                        type_convertion(x.aabb.mins),
                        type_convertion(x.aabb.maxs),
                        multiple_resolution_mesh_pass.transformation,
                    )
                })
                .collect();
            boxes.append(&mut aabbs);
        }

        let culling_results = self
            .box_culling_pipeline
            .execute(device, queue, global_constants_buffer, depth_view, &boxes)
            .ok()?;

        for (i, _) in draw_objects.iter().enumerate() {
            let range = &ranges[i];
            let visibility_indices = culling_results[range.clone()].to_vec();
            let cluster_ids = cluster_ids[range.clone()].to_vec();
            assert_eq!(visibility_indices.len(), cluster_ids.len());
            visible_cluster_ids.push(
                zip(visibility_indices, cluster_ids)
                    .filter(|(is_visible, _)| *is_visible == 1)
                    .map(|(_, cluster_id)| cluster_id)
                    .collect(),
            );
        }

        Some(visible_cluster_ids)
    }
}

fn type_convertion(value: rapier3d::math::Point<rapier3d::prelude::Real>) -> glam::Vec3 {
    glam::vec3(value.x, value.y, value.z)
}
