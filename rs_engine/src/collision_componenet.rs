use crate::{
    content::content_file_type::EContentFileType,
    drawable::{CustomDrawObject, EDrawObjectType},
    engine::Engine,
    player_viewport::PlayerViewport,
    scene_node::{EComponentType, SceneNode},
};
use rapier3d::prelude::*;
use rs_core_minimal::primitive_data::PrimitiveData;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::{
    command::{DrawObject, EBindingResource},
    constants,
    renderer::{EBuiltinPipelineType, EPipelineType},
    vertex_data_type::mesh_vertex::MeshVertex3,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Physics {
    pub colliders: Vec<Collider>,
    pub rigid_body: RigidBody,
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handles: Vec<ColliderHandle>,
}

impl Physics {
    pub fn get_collider_handles(&self) -> Vec<ColliderHandle> {
        self.collider_handles.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ECollisionType {
    Cube,
}

#[derive(Clone)]
pub struct CollisionComponentRuntime {
    pub physics: Option<Physics>,
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,

    draw_object: EDrawObjectType,
    constants_handle: crate::handle::BufferHandle,
    constants: constants::Constants,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CollisionComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    pub collision_type: ECollisionType,
    #[serde(skip)]
    pub run_time: Option<CollisionComponentRuntime>,
}

impl CollisionComponent {
    pub fn new_scene_node(
        name: String,
        transformation: glam::Mat4,
    ) -> SingleThreadMutType<SceneNode> {
        let collision_component = Self::new(name, transformation);
        let collision_component = SingleThreadMut::new(collision_component);
        let scene_node = SceneNode {
            component: EComponentType::CollisionComponent(collision_component),
            childs: vec![],
        };
        let scene_node = SingleThreadMut::new(scene_node);
        scene_node
    }

    pub fn new(name: String, transformation: glam::Mat4) -> CollisionComponent {
        CollisionComponent {
            name,
            transformation,
            run_time: None,
            collision_type: ECollisionType::Cube,
        }
    }

    pub fn get_final_transformation(&self) -> glam::Mat4 {
        self.run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default()
    }

    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_transformation(&self) -> &glam::Mat4 {
        &self.transformation
    }

    pub fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        let _ = files;
        match self.collision_type {
            ECollisionType::Cube => {
                let cube = PrimitiveData::cube_lines();
                let mut vertex: Vec<MeshVertex3> = Vec::with_capacity(cube.into_iter().count());

                for (_, vertex_position, ..) in cube.into_iter() {
                    vertex.push(MeshVertex3 {
                        position: *vertex_position,
                        vertex_color: glam::vec4(1.0, 0.0, 0.0, 1.0),
                    });
                }

                let vertex_count = vertex.len();
                let vertex_buffer_handle =
                    engine.create_vertex_buffer(&vertex, Some(format!("rs.VertexBuffer")));

                let index_buffer_handle =
                    engine.create_index_buffer(&cube.indices, Some(format!("rs.IndexBuffer")));

                let constants_handle = engine.create_constants_buffer(
                    &vec![constants::Constants::default()],
                    Some(format!("rs.Constants")),
                );

                let draw_object = DrawObject::new(
                    0,
                    vec![*vertex_buffer_handle],
                    vertex_count as u32,
                    EPipelineType::Builtin(EBuiltinPipelineType::Primitive),
                    Some(*index_buffer_handle),
                    Some(cube.indices.len() as u32),
                    vec![
                        vec![EBindingResource::Constants(
                            *player_viewport.global_constants_handle,
                        )],
                        vec![EBindingResource::Constants(*constants_handle)],
                    ],
                );
                self.run_time = Some(CollisionComponentRuntime {
                    physics: None,
                    parent_final_transformation: glam::Mat4::IDENTITY,
                    final_transformation: glam::Mat4::IDENTITY,
                    draw_object: EDrawObjectType::Custom(CustomDrawObject {
                        draw_object,
                        render_target_type: *player_viewport.get_render_target_type(),
                    }),
                    constants_handle,
                    constants: constants::Constants::default(),
                });
            }
        }
    }

    fn default_rad() -> f32 {
        1.0
    }

    fn build_physics(
        collision_type: &ECollisionType,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Physics> {
        let (scale, rotation, translation) = transformation.to_scale_rotation_translation();
        let collider_builder: ColliderBuilder = match collision_type {
            ECollisionType::Cube => {
                let rad = Self::default_rad();
                let hx = rad * scale.x;
                let hy = rad * scale.y;
                let hz = rad * scale.z;
                ColliderBuilder::cuboid(hx, hy, hz)
                    .sensor(true)
                    .density(0.0)
            }
        };
        let (axis, angle) = rotation.to_axis_angle();

        let mut sensor_builder = RigidBodyBuilder::fixed().translation(vector![
            translation.x,
            translation.y,
            translation.z
        ]);
        sensor_builder.position.rotation = Rotation::from_axis_angle(
            &UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]),
            angle,
        );
        let rigid_body = sensor_builder.build();
        let collider = collider_builder.build();

        Ok(Physics {
            colliders: vec![collider],
            rigid_body,
            rigid_body_handle: RigidBodyHandle::invalid(),
            collider_handles: vec![],
        })
    }

    pub fn initialize_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        let Ok(mut physics) =
            Self::build_physics(&self.collision_type, run_time.final_transformation)
        else {
            return;
        };
        let handle = rigid_body_set.insert(physics.rigid_body.clone());
        for collider in physics.colliders.clone() {
            let collider_handle = collider_set.insert_with_parent(collider, handle, rigid_body_set);
            physics.collider_handles.push(collider_handle);
        }
        physics.rigid_body_handle = handle;

        run_time.physics = Some(physics);
    }

    pub fn recreate_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        self.initialize_physics(rigid_body_set, collider_set);
    }

    pub fn get_draw_objects(&self) -> Vec<&crate::drawable::EDrawObjectType> {
        self.run_time
            .as_ref()
            .map(|x| vec![&x.draw_object])
            .unwrap_or(vec![])
    }

    pub fn tick(
        &mut self,
        time: f32,
        engine: &mut Engine,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let _ = collider_set;
        let _ = rigid_body_set;
        let _ = time;
        if let Some(run_time) = self.run_time.as_mut() {
            run_time.constants.model = run_time.final_transformation;
            engine.update_buffer(
                run_time.constants_handle.clone(),
                rs_foundation::cast_any_as_u8_slice(&run_time.constants),
            );
        }
    }

    pub fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::Physics>,
    ) {
        let Some(level_physics) = level_physics else {
            return;
        };
        self.recreate_physics(
            &mut level_physics.rigid_body_set,
            &mut level_physics.collider_set,
        );
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut Physics> {
        self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten()
    }

    pub fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    pub fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    pub fn get_physics(&self) -> Option<&Physics> {
        self.run_time.as_ref().map(|x| x.physics.as_ref()).flatten()
    }

    // pub fn get_shape<'a>(&self, collider_set: &'a ColliderSet) -> Option<&'a dyn Shape> {
    //     let Some(run_time) = &self.run_time else {
    //         return None;
    //     };
    //     let Some(physics) = run_time.physics.as_ref() else {
    //         return None;
    //     };

    //     let collider = &collider_set[physics.collider_handles[0]];
    //     Some(collider.shape())
    // }

    pub fn get_collider_handle(&self) -> Option<ColliderHandle> {
        let Some(run_time) = &self.run_time else {
            return None;
        };
        let Some(physics) = run_time.physics.as_ref() else {
            return None;
        };
        physics.collider_handles.get(0).cloned()
    }
}
