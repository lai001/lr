use crate::{engine::Engine, handle::BufferHandle, resource_manager::ResourceManager};
use glam::Vec4Swizzles;
use rs_core_minimal::primitive_data::PrimitiveData;
use rs_render::{
    command::{BufferCreateInfo, CreateBuffer, Draw, DrawObject, EBindingResource, EDrawCallType},
    renderer::{EBuiltinPipelineType, EPipelineType},
    vertex_data_type::mesh_vertex::{Instance0, MeshVertex0},
};
use wgpu::*;

pub struct EmiterRender {
    vertex_buffer_handle: BufferHandle,
    index_buffer_handle: BufferHandle,
    global_constants_handle: BufferHandle,
}

impl EmiterRender {
    pub fn new(engine: &mut Engine, global_constants_handle: BufferHandle) -> EmiterRender {
        let rm = ResourceManager::default();
        let vertex_buffer_handle = rm.next_buffer();
        let index_buffer_handle = rm.next_buffer();

        let quad = PrimitiveData::quad();

        let command = rs_render::command::RenderCommand::CreateBuffer(CreateBuffer {
            handle: *vertex_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some(format!("VertexBuffer")),
                contents: rs_foundation::cast_to_raw_buffer(
                    &quad
                        .into_iter()
                        .map(|x| MeshVertex0 {
                            position: (glam::Mat4::from_rotation_x(90_f32.to_radians())
                                * glam::vec4(x.1.x, x.1.y, x.1.z, 1.0))
                            .xyz(),
                            tex_coord: *x.5,
                        })
                        .collect::<Vec<MeshVertex0>>(),
                )
                .to_vec(),
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            },
        });
        engine.send_render_command(command);

        let command = rs_render::command::RenderCommand::CreateBuffer(CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some(format!("IndexBuffer")),
                contents: rs_foundation::cast_to_raw_buffer(&quad.indices).to_vec(),
                usage: BufferUsages::INDEX,
            },
        });
        engine.send_render_command(command);

        EmiterRender {
            vertex_buffer_handle,
            index_buffer_handle,
            global_constants_handle,
        }
    }

    pub fn collect_emiter_render(
        &self,
        particle_system: &crate::particle::system::ParticleSystem,
        engine: &mut Engine,
    ) -> Vec<DrawObject> {
        let rm = ResourceManager::default();
        let quad = PrimitiveData::quad();
        let mut draw_objects = vec![];
        let mut position_colors: Vec<(glam::Vec3, glam::Vec4)> = vec![];
        for (_, emiter) in &particle_system.emiters {
            match emiter {
                crate::particle::emiter::ParticleEmiter::Spawn(emiter) => {
                    position_colors.append(&mut emiter.get_parameters());
                }
            }
        }
        if position_colors.is_empty() {
            return vec![];
        }
        let instances: Vec<Instance0> = position_colors
            .iter()
            .map(|(position, color)| Instance0 {
                position: *position,
                color: *color,
            })
            .collect();
        let instance_buffer_handle = rm.next_buffer();
        let command = rs_render::command::RenderCommand::CreateBuffer(CreateBuffer {
            handle: *instance_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some(format!("InstanceBuffer")),
                contents: rs_foundation::cast_to_raw_buffer(&instances).to_vec(),
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            },
        });
        engine.send_render_command(command);
        let mut draw_object = DrawObject::new(
            0,
            vec![*self.vertex_buffer_handle, *instance_buffer_handle],
            quad.vertex_positions.len() as u32,
            EPipelineType::Builtin(EBuiltinPipelineType::Particle),
            Some(*self.index_buffer_handle),
            Some(quad.indices.len() as u32),
            vec![vec![EBindingResource::Constants(
                *self.global_constants_handle,
            )]],
        );
        draw_object.draw_call_type = EDrawCallType::Draw(Draw {
            instances: 0..(instances.len() as u32),
        });

        draw_objects.push(draw_object);
        draw_objects
    }
}
