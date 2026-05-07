use crate::{
    content::{content_file_type::EContentFileType, level::LevelPhysics},
    engine::Engine,
    resource_manager::ResourceManager,
};
use rapier3d::{
    control::KinematicCharacterController,
    parry::shape::Ball,
    prelude::{
        ActiveEvents, Collider, ColliderBuilder, ColliderHandle, Cuboid, HalfSpace, QueryFilter,
        RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodyType, SharedShape, TriMeshFlags,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeshOptions {
    pub mesh_url: Option<url::Url>,
    pub is_use_convex_decomposition: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EShapeType {
    HalfSpace(HalfSpace),
    Ball(Ball),
    Cuboid(Cuboid),
    Mesh(MeshOptions),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Initialization {
    pub rigid_body_type: RigidBodyType,
    pub shape_type: EShapeType,
}

#[derive(Clone)]
pub struct PhysicsAbility {
    pub colliders: Vec<Collider>,
    pub collider_handles: Vec<ColliderHandle>,
    pub rigid_body: RigidBody,
    pub rigid_body_handle: RigidBodyHandle,
    pub is_apply_simulate: bool,
    pub controller: Option<KinematicCharacterController>,
    pub scale: glam::Vec3,
    translation: glam::Vec3,
    rotation: glam::Quat,
    controller_desired_movement: glam::Vec3,
}

impl PhysicsAbility {
    pub fn is_valid(&self) -> bool {
        if self.rigid_body_handle == RigidBodyHandle::invalid() {
            return false;
        }
        if self.collider_handles.len() != self.colliders.len() || self.collider_handles.is_empty() {
            return false;
        }
        for handle in &self.collider_handles {
            if *handle == ColliderHandle::invalid() {
                return false;
            }
        }
        return true;
    }

    fn shared_shape(
        initialization: &Initialization,
        files: &[EContentFileType],
        resource_manager: ResourceManager,
    ) -> std::result::Result<SharedShape, String> {
        let shape: std::result::Result<SharedShape, String> = match &initialization.shape_type {
            EShapeType::HalfSpace(half_space) => Ok(SharedShape::halfspace(half_space.normal)),
            EShapeType::Ball(ball) => Ok(SharedShape::ball(ball.radius)),
            EShapeType::Cuboid(cuboid) => Ok(SharedShape::cuboid(
                cuboid.half_extents.x,
                cuboid.half_extents.y,
                cuboid.half_extents.z,
            )),
            EShapeType::Mesh(MeshOptions {
                mesh_url,
                is_use_convex_decomposition,
            }) => {
                let shape: std::result::Result<SharedShape, String>;
                let mut static_mesh: Option<Arc<rs_artifact::static_mesh::StaticMesh>> = None;

                for file in files {
                    if let EContentFileType::StaticMesh(mesh) = file {
                        let mesh = mesh.borrow();
                        if Some(mesh.url.clone()) == *mesh_url {
                            static_mesh = resource_manager
                                .get_static_mesh(&mesh.asset_info.get_url())
                                .ok();
                            break;
                        }
                    }
                }
                if let Some(static_mesh) = static_mesh {
                    shape = Self::shape_from_mesh(&static_mesh, *is_use_convex_decomposition)
                        .map_err(|err| format!("{}, url: {:?}", err.to_string(), *mesh_url));
                } else {
                    let mut source_urls = vec![];
                    for file in files {
                        if let EContentFileType::StaticMesh(mesh) = file {
                            let mesh = mesh.borrow();
                            if Some(mesh.url.clone()) == *mesh_url {
                                source_urls.push(mesh.asset_info.get_url());
                            }
                        }
                    }
                    shape = Err(format!(
                        "url: {:?}, source_urls: {:?}",
                        mesh_url, source_urls
                    ));
                }
                shape
            }
        };
        shape
    }

    pub fn new(
        initialization: &Initialization,
        transformation: glam::Mat4,
        is_apply_simulate: bool,
        files: &[EContentFileType],
        resource_manager: ResourceManager,
        level_physics: &mut LevelPhysics,
    ) -> PhysicsAbility {
        let (_, rotation, translation) = transformation.to_scale_rotation_translation();
        let (axis, angle) = rotation.to_axis_angle();

        let mut controller: Option<KinematicCharacterController> = None;
        let mut rigid_body_builder: RigidBodyBuilder;
        let mut colliders: Vec<Collider> = vec![];

        let shape = Self::shared_shape(initialization, files, resource_manager);
        match shape {
            Ok(shape) => {
                let collider_builder = ColliderBuilder::new(shape)
                    .contact_skin(0.1)
                    .active_events(ActiveEvents::COLLISION_EVENTS);
                let collider = collider_builder.build();
                colliders.push(collider);
            }
            Err(err) => {
                log::warn!("No shape, {}", err);
            }
        }

        match initialization.rigid_body_type {
            RigidBodyType::Dynamic => {
                rigid_body_builder = RigidBodyBuilder::dynamic().translation(translation);
            }
            RigidBodyType::Fixed => {
                rigid_body_builder = RigidBodyBuilder::fixed().translation(translation);
            }
            RigidBodyType::KinematicPositionBased => {
                let impossible_slope_angle = 0.6;
                rigid_body_builder = RigidBodyBuilder::kinematic_position_based()
                    .soft_ccd_prediction(10.0)
                    .translation(translation);

                controller = Some(KinematicCharacterController {
                    max_slope_climb_angle: impossible_slope_angle - 0.02,
                    min_slope_slide_angle: impossible_slope_angle - 0.02,
                    slide: true,
                    ..Default::default()
                });
            }
            RigidBodyType::KinematicVelocityBased => {
                unimplemented!()
            }
        }

        rigid_body_builder.position.rotation =
            rapier3d::math::Rotation::from_axis_angle(axis.normalize(), angle);

        let rigid_body = rigid_body_builder.build();

        let mut collider_handles: Vec<ColliderHandle> = Vec::with_capacity(colliders.len());
        let rigid_body_handle = level_physics.rigid_body_set.insert(rigid_body.clone());
        for collider in colliders.clone() {
            let collider_handle = level_physics.collider_set.insert_with_parent(
                collider,
                rigid_body_handle,
                &mut level_physics.rigid_body_set,
            );
            collider_handles.push(collider_handle);
        }

        PhysicsAbility {
            colliders,
            collider_handles,
            rigid_body,
            rigid_body_handle,
            is_apply_simulate,
            controller,
            scale: glam::Vec3::ONE,
            translation,
            rotation,
            controller_desired_movement: glam::Vec3::ZERO,
        }
    }

    pub fn collider_handles(&self) -> Vec<ColliderHandle> {
        self.collider_handles.clone()
    }

    pub fn shape_from_mesh(
        mesh: &rs_artifact::static_mesh::StaticMesh,
        is_use_convex_decomposition: bool,
    ) -> crate::error::Result<SharedShape> {
        let vertices: Vec<_> = mesh.vertexes.iter().map(|x| x.position).collect();
        // let deltas = Isometry::identity();
        // let aabb = bounding_volume::details::point_cloud_aabb(&deltas, &vertices);
        // let center = aabb.center();
        // let diag = (aabb.maxs - aabb.mins).norm();
        // vertices
        //     .iter_mut()
        //     .for_each(|p| *p = (*p - center.coords) * 10.0 / diag);

        let mut indices: Vec<_> = vec![];
        for index in mesh.indexes.chunks(3) {
            indices
                .push(<[u32; 3]>::try_from(index).map_err(crate::error::Error::TryFromSliceError)?);
        }

        let decomposed_shape = if is_use_convex_decomposition {
            SharedShape::convex_decomposition(&vertices, &indices)
        } else {
            SharedShape::trimesh_with_flags(vertices, indices, TriMeshFlags::FIX_INTERNAL_EDGES)
                .map_err(|err| {
                    crate::error::Error::Other(Some(format!("Fail to build mesh, {}", err)))
                })?
        };
        Ok(decomposed_shape)
    }

    pub fn collider_from_mesh(
        mesh: &rs_artifact::static_mesh::StaticMesh,
        is_use_convex_decomposition: bool,
    ) -> crate::error::Result<Collider> {
        let decomposed_shape = Self::shape_from_mesh(mesh, is_use_convex_decomposition)?;
        let collider = ColliderBuilder::new(decomposed_shape)
            .contact_skin(0.1)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();
        Ok(collider)
    }

    pub fn rotation(&self) -> &glam::Quat {
        &self.rotation
    }

    pub fn translation(&self) -> &glam::Vec3 {
        &self.translation
    }

    pub fn set_controller_desired_movement(&mut self, controller_desired_movement: glam::Vec3) {
        if let Some(_) = &self.controller {
            self.controller_desired_movement = controller_desired_movement;
        }
    }

    pub fn is_controller(&self) -> bool {
        self.controller.is_some()
    }

    pub fn tick(
        &mut self,
        delta_seconds: f32,
        engine: &mut Engine,
        level_physics: &mut LevelPhysics,
    ) {
        if !self.is_apply_simulate {
            return;
        }

        if let Some(controller) = &mut self.controller {
            debug_assert_eq!(self.collider_handles.len(), 1);
            let character_collider = &level_physics.collider_set[self.collider_handles[0]];
            let character_pose = *character_collider.position();
            let character_shape = character_collider.shared_shape().clone();
            let character_mass = level_physics.rigid_body_set[self.rigid_body_handle].mass();

            let mut collisions = vec![];

            let mut query_pipeline = level_physics.query_pipeline_mut(Some(
                QueryFilter::new().exclude_rigid_body(self.rigid_body_handle),
            ));
            let possible_movement = controller.move_shape(
                delta_seconds,
                &query_pipeline.as_ref(),
                &*character_shape,
                &character_pose,
                self.controller_desired_movement,
                |c| collisions.push(c),
            );

            controller.solve_character_collision_impulses(
                delta_seconds,
                &mut query_pipeline,
                &*character_shape,
                character_mass,
                &*collisions,
            );

            let character_body = &mut level_physics.rigid_body_set[self.rigid_body_handle];
            let pose = character_body.position();
            self.translation = pose.translation;
            self.rotation = pose.rotation;
            character_body
                .set_next_kinematic_translation(pose.translation + possible_movement.translation);
        } else {
            let _ = level_physics;
            let _ = engine;
            let _ = delta_seconds;

            let rigid_body = &level_physics.rigid_body_set[self.rigid_body_handle];
            self.translation = rigid_body.translation();
            self.rotation = *rigid_body.rotation();
        }
    }
}
