use crate::{
    content::content_file_type::EContentFileType, drawable::EDrawObjectType, engine::Engine,
    resource_manager::ResourceManager,
};
use rapier3d::prelude::*;
use rs_artifact::static_mesh::StaticMesh;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct Physics {
    collider: Collider,
    pub rigid_body: RigidBody,
    rigid_body_handle: RigidBodyHandle,
    is_apply_simulate: bool,
}

#[derive(Clone)]
pub struct StaticMeshComponentRuntime {
    draw_objects: EDrawObjectType,
    _mesh: Arc<StaticMesh>,
    pub physics: Option<Physics>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StaticMeshComponent {
    pub name: String,
    pub static_mesh: Option<url::Url>,
    pub transformation: glam::Mat4,
    pub material_url: Option<url::Url>,

    #[serde(skip)]
    pub run_time: Option<StaticMeshComponentRuntime>,
}

impl StaticMeshComponent {
    pub fn get_interactive_transformation(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn new(
        name: String,
        static_mesh_url: Option<url::Url>,
        material_url: Option<url::Url>,
        transformation: glam::Mat4,
    ) -> StaticMeshComponent {
        StaticMeshComponent {
            name,
            transformation,
            material_url,
            run_time: None,
            static_mesh: static_mesh_url,
        }
    }

    pub fn initialize(
        &mut self,
        resource_manager: ResourceManager,
        engine: &mut Engine,
        files: &[EContentFileType],
    ) {
        let mut find_static_mesh: Option<Arc<StaticMesh>> = None;

        for file in files {
            if let EContentFileType::StaticMesh(mesh) = file {
                let mesh = mesh.borrow();
                if Some(mesh.url.clone()) == self.static_mesh {
                    find_static_mesh = resource_manager
                        .get_static_mesh(&mesh.asset_info.get_url())
                        .ok();
                    break;
                }
            }
        }

        let material = if let Some(material_url) = &self.material_url {
            files.iter().find_map(|x| {
                if let EContentFileType::Material(content_material) = x {
                    if &content_material.borrow().url == material_url {
                        return Some(content_material.clone());
                    }
                }
                None
            })
        } else {
            None
        };

        if let Some(find_static_mesh) = find_static_mesh {
            let mut draw_object: EDrawObjectType;
            if let Some(material) = material.clone() {
                draw_object = engine.create_material_draw_object_from_static_mesh(
                    &find_static_mesh.vertexes,
                    &find_static_mesh.indexes,
                    Some(find_static_mesh.name.clone()),
                    material,
                );
            } else {
                draw_object = engine.create_draw_object_from_static_mesh(
                    &find_static_mesh.vertexes,
                    &find_static_mesh.indexes,
                    Some(find_static_mesh.name.clone()),
                );
            }
            match &mut draw_object {
                EDrawObjectType::Static(draw_object) => {
                    draw_object.constants.model = self.transformation;
                }
                EDrawObjectType::StaticMeshMaterial(draw_object) => {
                    draw_object.constants.model = self.transformation;
                }
                _ => unimplemented!(),
            }
            let physics =
                Self::build_physics(find_static_mesh.as_ref(), false, self.transformation).ok();
            self.run_time = Some(StaticMeshComponentRuntime {
                draw_objects: draw_object,
                _mesh: find_static_mesh,
                physics,
            })
        }
    }

    pub fn update(
        &mut self,
        time: f32,
        engine: &mut Engine,
        mut rigid_body_set: Option<&mut RigidBodySet>,
    ) {
        let _ = time;
        let _ = engine;
        let Some(run_time) = &mut self.run_time else {
            return;
        };

        let is_simulate = run_time
            .physics
            .as_mut()
            .map(|x| x.is_apply_simulate)
            .unwrap_or(false);

        match (
            run_time.physics.as_mut(),
            rigid_body_set.as_mut(),
            is_simulate,
        ) {
            (Some(physics), Some(rigid_body_set), true) => {
                let rigid_body = &rigid_body_set[physics.rigid_body_handle];
                let translation = rigid_body.translation();
                let translation = glam::vec3(translation.x, translation.y, translation.z);
                let rotation = rigid_body.rotation();
                let rotation = glam::quat(rotation.i, rotation.j, rotation.k, rotation.w);
                let scale = self.transformation.to_scale_rotation_translation().0;
                let transformation =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
                match &mut run_time.draw_objects {
                    EDrawObjectType::Static(draw_object) => {
                        draw_object.constants.model = transformation;
                    }
                    EDrawObjectType::StaticMeshMaterial(draw_object) => {
                        draw_object.constants.model = transformation;
                    }
                    _ => unimplemented!(),
                }
            }
            _ => {
                let transformation = self.transformation;
                match &mut run_time.draw_objects {
                    EDrawObjectType::Static(draw_object) => {
                        draw_object.constants.model = transformation;
                    }
                    EDrawObjectType::StaticMeshMaterial(draw_object) => {
                        draw_object.constants.model = transformation;
                    }
                    _ => unimplemented!(),
                }
            }
        }

        engine.update_draw_object(&mut run_time.draw_objects);
    }

    pub fn get_draw_objects(&self) -> Vec<&EDrawObjectType> {
        match &self.run_time {
            Some(x) => vec![&x.draw_objects],
            None => vec![],
        }
    }

    pub fn get_draw_objects_mut(&mut self) -> Vec<&mut EDrawObjectType> {
        match &mut self.run_time {
            Some(x) => vec![&mut x.draw_objects],
            None => vec![],
        }
    }

    pub fn set_material(
        &mut self,
        engine: &mut Engine,
        new_material_url: Option<url::Url>,
        files: &[EContentFileType],
    ) {
        self.material_url = new_material_url;
        let material = if let Some(material_url) = &self.material_url {
            files.iter().find_map(|x| {
                if let EContentFileType::Material(content_material) = x {
                    if &content_material.borrow().url == material_url {
                        return Some(content_material.clone());
                    }
                }
                None
            })
        } else {
            None
        };

        if let Some(run_time) = self.run_time.as_mut() {
            let static_mesh = run_time._mesh.clone();
            let draw_object: EDrawObjectType;
            if let Some(material) = material.clone() {
                draw_object = engine.create_material_draw_object_from_static_mesh(
                    &static_mesh.vertexes,
                    &static_mesh.indexes,
                    Some(static_mesh.name.clone()),
                    material,
                );
            } else {
                draw_object = engine.create_draw_object_from_static_mesh(
                    &static_mesh.vertexes,
                    &static_mesh.indexes,
                    Some(static_mesh.name.clone()),
                );
            }
            run_time.draw_objects = draw_object;
        }
    }

    fn build_physics(
        mesh: &StaticMesh,
        is_use_convex_decomposition: bool,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Physics> {
        let vertices: Vec<_> = mesh
            .vertexes
            .iter()
            .map(|x| point![x.position.x, x.position.y, x.position.z])
            .collect();
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
        };
        let (_, rotation, translation) = transformation.to_scale_rotation_translation();
        let translation = vector![translation.x, translation.y, translation.z];
        let (axis, angle) = rotation.to_axis_angle();
        let collider = ColliderBuilder::new(decomposed_shape)
            .contact_skin(0.1)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();
        let mut builder = RigidBodyBuilder::dynamic();
        builder = builder.translation(translation);
        builder.position.rotation = Rotation::from_axis_angle(
            &UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]),
            angle,
        );
        // builder = builder.enabled_rotations(false, false, false);
        let rigid_body = builder.build();

        Ok(Physics {
            collider,
            rigid_body,
            rigid_body_handle: RigidBodyHandle::invalid(),
            is_apply_simulate: true,
        })
    }

    pub fn init_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        let handle = rigid_body_set.insert(physics.rigid_body.clone());
        collider_set.insert_with_parent(physics.collider.clone(), handle, rigid_body_set);
        physics.rigid_body_handle = handle;
    }

    pub fn update_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        let Some(rigid_body) = rigid_body_set.get_mut(physics.rigid_body_handle) else {
            return;
        };
        for handle in rigid_body.colliders() {
            if let Some(collider) = collider_set.get_mut(*handle) {
                collider.copy_from(&physics.collider);
            }
        }
        rigid_body.copy_from(&physics.rigid_body);
    }

    pub fn set_apply_simulate(&mut self, is_apply_simulate: bool) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        physics.is_apply_simulate = is_apply_simulate;
    }

    pub fn on_post_update_transformation(&mut self, rigid_body_set: Option<&mut RigidBodySet>) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        let Some(rigid_body_set) = rigid_body_set else {
            return;
        };
        let Some(rigid_body) = rigid_body_set.get_mut(physics.rigid_body_handle) else {
            return;
        };

        let (_, rotation, translation) = self.transformation.to_scale_rotation_translation();
        let translation = vector![translation.x, translation.y, translation.z];
        rigid_body.set_translation(translation, false);
        let (axis, angle) = rotation.to_axis_angle();
        rigid_body.set_rotation(
            Rotation::from_axis_angle(
                &UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]),
                angle,
            ),
            false,
        );
        rigid_body.set_angvel(vector![0.0, 0.0, 0.0], false);
        rigid_body.set_linvel(vector![0.0, 0.0, 0.0], false);
        rigid_body.reset_forces(false);
        rigid_body.reset_torques(false);
        rigid_body.wake_up(true);
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut Physics> {
        self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten()
    }
}
