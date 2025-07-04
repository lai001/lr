#[cfg(feature = "network")]
use crate::network;
#[cfg(feature = "network")]
use crate::network::NetworkReplicated;
use crate::{
    content::{content_file_type::EContentFileType, material::Material},
    drawable::EDrawObjectType,
    engine::Engine,
    misc::{static_mesh_get_aabb, transform_aabb},
    player_viewport::PlayerViewport,
    resource_manager::ResourceManager,
};
use rapier3d::prelude::*;
use rs_artifact::static_mesh::StaticMesh;
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};
#[cfg(feature = "network")]
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Physics {
    pub colliders: Vec<Collider>,
    pub rigid_body: RigidBody,
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handles: Vec<ColliderHandle>,
    pub is_apply_simulate: bool,
}

impl Physics {
    pub fn get_collider_handles(&self) -> Vec<ColliderHandle> {
        self.collider_handles.clone()
    }
}

#[derive(Clone)]
pub struct StaticMeshComponentRuntime {
    draw_objects: Option<EDrawObjectType>,
    _mesh: Option<Arc<StaticMesh>>,
    pub physics: Option<Physics>,
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    aabb: Option<Aabb>,
}

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum ReplicatedFieldType {
    IsVisible,
    Transformation,
}

#[cfg(feature = "network")]
type TransmissionType = HashMap<ReplicatedFieldType, Vec<u8>>;

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NetworkFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    net_id: Option<uuid::Uuid>,
    #[serde(default = "bool::default")]
    pub is_replicated: bool,
    #[serde(skip)]
    replicated_datas: TransmissionType,
}

#[cfg(feature = "network")]
impl NetworkFields {
    pub fn new() -> NetworkFields {
        NetworkFields {
            net_id: Some(network::default_uuid()),
            is_replicated: false,
            replicated_datas: TransmissionType::new(),
        }
    }

    pub fn set_is_visible(&mut self, is_visible: bool) -> rs_artifact::error::Result<()> {
        let data = rs_artifact::bincode_legacy::serialize(&is_visible, None)?;
        self.replicated_datas
            .insert(ReplicatedFieldType::IsVisible, data);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.replicated_datas.drain();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StaticMeshComponent {
    pub name: String,
    pub static_mesh: Option<url::Url>,
    pub transformation: glam::Mat4,
    pub material_url: Option<url::Url>,
    pub is_visible: bool,
    pub rigid_body_type: RigidBodyType,
    pub is_enable_multiresolution: bool,
    #[cfg(feature = "network")]
    #[serde(default)]
    pub network_fields: NetworkFields,
    #[serde(skip)]
    pub run_time: Option<StaticMeshComponentRuntime>,
}

#[cfg(feature = "network")]
impl crate::network::NetworkReplicated for StaticMeshComponent {
    fn get_network_id(&self) -> &uuid::Uuid {
        self.network_fields.net_id.as_ref().expect("A valid id")
    }

    fn set_network_id(&mut self, network_id: uuid::Uuid) {
        self.network_fields.net_id = Some(network_id);
    }

    fn is_replicated(&self) -> bool {
        self.network_fields.is_replicated
    }

    fn set_replicated(&mut self, is_replicated: bool) {
        self.network_fields.is_replicated = is_replicated;
    }

    fn on_replicated(&mut self) -> Vec<u8> {
        if self.network_fields.replicated_datas.is_empty() {
            return vec![];
        }
        let encoded_data = (|| {
            rs_artifact::bincode_legacy::serialize::<TransmissionType>(
                &self.network_fields.replicated_datas,
                None,
            )
        })();
        if let Err(err) = &encoded_data {
            log::warn!("{}", err);
        }
        self.network_fields.reset();
        encoded_data.unwrap_or_default()
    }

    fn on_sync(&mut self, data: &Vec<u8>) {
        let sync_result: rs_artifact::error::Result<()> = (|| {
            let decoded_data =
                rs_artifact::bincode_legacy::deserialize::<TransmissionType>(&data, None)?;
            for (k, v) in decoded_data {
                match k {
                    ReplicatedFieldType::Transformation => {
                        self.transformation =
                            rs_artifact::bincode_legacy::deserialize::<glam::Mat4>(&v, None)?;
                    }
                    ReplicatedFieldType::IsVisible => {
                        self.is_visible =
                            rs_artifact::bincode_legacy::deserialize::<bool>(&v, None)?;
                    }
                }
            }
            Ok(())
        })();
        if let Err(err) = &sync_result {
            log::warn!("{}", err);
        }
    }

    fn debug_name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}

impl StaticMeshComponent {
    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_transformation(&self) -> &glam::Mat4 {
        &self.transformation
    }

    pub fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    pub fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    pub fn get_final_transformation(&self) -> glam::Mat4 {
        self.run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default()
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
            is_visible: true,
            rigid_body_type: RigidBodyType::Dynamic,
            is_enable_multiresolution: false,
            #[cfg(feature = "network")]
            network_fields: NetworkFields::new(),
        }
    }

    pub fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        #[cfg(feature = "network")]
        if self.network_fields.net_id.is_none() {
            self.set_network_id(crate::network::default_uuid());
        }
        let resource_manager = engine.get_resource_manager();
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
                    Some(format!("{} - {}", &self.name, &find_static_mesh.name)),
                    material,
                    player_viewport.global_constants_handle.clone(),
                    player_viewport.point_lights_constants_handle.clone(),
                    player_viewport.spot_lights_constants_handle.clone(),
                );
            } else {
                draw_object = engine.create_draw_object_from_static_mesh(
                    &find_static_mesh.vertexes,
                    &find_static_mesh.indexes,
                    Some(find_static_mesh.name.clone()),
                    player_viewport.global_constants_handle.clone(),
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
            let aabb = static_mesh_get_aabb(&find_static_mesh);
            self.run_time = Some(StaticMeshComponentRuntime {
                draw_objects: Some(draw_object),
                _mesh: Some(find_static_mesh),
                physics: None,
                final_transformation: glam::Mat4::IDENTITY,
                parent_final_transformation: glam::Mat4::IDENTITY,
                aabb: Some(aabb),
            });
            self.on_is_enable_multiresolution_changed();
        }
    }

    pub fn tick(
        &mut self,
        time: f32,
        engine: &mut Engine,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let _ = collider_set;
        let _ = time;
        let _ = engine;
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        let Some(draw_objects) = run_time.draw_objects.as_mut() else {
            return;
        };

        let is_simulate = run_time
            .physics
            .as_mut()
            .map(|x| x.is_apply_simulate)
            .unwrap_or(false);

        match (
            run_time.physics.as_mut(),
            // rigid_body_set.as_mut(),
            is_simulate,
        ) {
            (Some(physics), true) => {
                let rigid_body = &rigid_body_set[physics.rigid_body_handle];
                let translation = rigid_body.translation();
                let translation = glam::vec3(translation.x, translation.y, translation.z);
                let rotation = rigid_body.rotation();
                let rotation = glam::quat(rotation.i, rotation.j, rotation.k, rotation.w);
                let scale = run_time
                    .final_transformation
                    .to_scale_rotation_translation()
                    .0;
                let transformation =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
                match draw_objects {
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
                let transformation = run_time.final_transformation;
                match draw_objects {
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
    }

    pub fn get_draw_objects(&self) -> Vec<&EDrawObjectType> {
        if !self.is_visible {
            return vec![];
        }
        match &self.run_time {
            Some(x) => match &x.draw_objects {
                Some(draw_objects) => vec![draw_objects],
                None => vec![],
            },
            None => vec![],
        }
    }

    pub fn get_draw_objects_mut(&mut self) -> Vec<&mut EDrawObjectType> {
        if !self.is_visible {
            return vec![];
        }
        match &mut self.run_time {
            Some(x) => match &mut x.draw_objects {
                Some(draw_objects) => vec![draw_objects],
                None => vec![],
            },
            None => vec![],
        }
    }

    pub fn set_material(
        &mut self,
        engine: &mut Engine,
        new_material_url: Option<url::Url>,
        files: &[EContentFileType],

        player_viewport: &mut PlayerViewport,
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
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        let Some(static_mesh) = run_time._mesh.as_ref() else {
            return;
        };

        let draw_object: EDrawObjectType;
        if let Some(material) = material.clone() {
            draw_object = engine.create_material_draw_object_from_static_mesh(
                &static_mesh.vertexes,
                &static_mesh.indexes,
                Some(format!("{} - {}", &self.name, &static_mesh.name)),
                material,
                player_viewport.global_constants_handle.clone(),
                player_viewport.point_lights_constants_handle.clone(),
                player_viewport.spot_lights_constants_handle.clone(),
            );
        } else {
            draw_object = engine.create_draw_object_from_static_mesh(
                &static_mesh.vertexes,
                &static_mesh.indexes,
                Some(static_mesh.name.clone()),
                player_viewport.global_constants_handle.clone(),
            );
        }
        run_time.draw_objects = Some(draw_object);
    }

    fn build_physics(
        mesh: &StaticMesh,
        is_use_convex_decomposition: bool,
        transformation: glam::Mat4,
        rigid_body_type: RigidBodyType,
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
                .map_err(|err| {
                    crate::error::Error::Other(Some(format!("Fail to build mesh, {}", err)))
                })?
        };
        let (_, rotation, translation) = transformation.to_scale_rotation_translation();
        let translation = vector![translation.x, translation.y, translation.z];
        let (axis, angle) = rotation.to_axis_angle();
        let collider = ColliderBuilder::new(decomposed_shape)
            .contact_skin(0.1)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let mut builder = match rigid_body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::KinematicPositionBased => RigidBodyBuilder::kinematic_position_based(),
            RigidBodyType::KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based(),
        };
        builder = builder.translation(translation);
        builder.position.rotation = Rotation::from_axis_angle(
            &UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]),
            angle,
        );
        // builder = builder.enabled_rotations(false, false, false);
        let rigid_body = builder.build();

        Ok(Physics {
            colliders: vec![collider],
            rigid_body,
            rigid_body_handle: RigidBodyHandle::invalid(),
            is_apply_simulate: true,
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
        let Some(static_mesh) = run_time._mesh.as_ref() else {
            return;
        };
        let Ok(mut physics) = Self::build_physics(
            static_mesh,
            false,
            run_time.final_transformation,
            self.rigid_body_type.clone(),
        ) else {
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

    pub fn set_apply_simulate(&mut self, is_apply_simulate: bool) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        physics.is_apply_simulate = is_apply_simulate;
    }

    pub fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::Physics>,
    ) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };

        let Some(physics) = run_time.physics.as_mut() else {
            return;
        };
        let Some(level_physics) = level_physics else {
            return;
        };

        let rigid_body = level_physics
            .rigid_body_set
            .get_mut(physics.rigid_body_handle)
            .unwrap();
        let collider = level_physics
            .collider_set
            .get_mut(physics.collider_handles[0])
            .unwrap();

        let (_, rotation, translation) = run_time
            .final_transformation
            .to_scale_rotation_translation();
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

        collider.set_position(translation.into());
        collider.set_rotation(Rotation::from_axis_angle(
            &UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]),
            angle,
        ));
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut Physics> {
        self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten()
    }

    pub fn get_physics(&self) -> Option<&Physics> {
        self.run_time.as_ref().map(|x| x.physics.as_ref()).flatten()
    }

    pub fn get_aabb(&self) -> Option<Aabb> {
        self.run_time
            .as_ref()
            .map(|x| {
                if let Some(aabb) = &x.aabb {
                    Some(transform_aabb(aabb, &self.get_final_transformation()))
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn set_static_mesh_url(
        &mut self,
        static_mesh_url: Option<url::Url>,
        resource_manager: ResourceManager,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &PlayerViewport,
    ) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        self.static_mesh = static_mesh_url;

        if self.static_mesh.is_none() {
            run_time._mesh = None;
            run_time.aabb = None;
            run_time.draw_objects = None;
            run_time.physics = None;
            return;
        }

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

        let Some(find_static_mesh) = find_static_mesh else {
            return;
        };

        let mut existing_material: Option<SingleThreadMutType<Material>> = None;
        if let Some(draw_objects) = &run_time.draw_objects {
            match draw_objects {
                EDrawObjectType::SkinMaterial(material_draw_object) => {
                    existing_material = Some(material_draw_object.material.clone());
                }
                EDrawObjectType::StaticMeshMaterial(static_mesh_material_draw_object) => {
                    existing_material = Some(static_mesh_material_draw_object.material.clone());
                }
                _ => {}
            }
        }
        if existing_material.is_none() {
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
            existing_material = material;
        }

        let mut draw_object: EDrawObjectType;
        if let Some(material) = existing_material.clone() {
            draw_object = engine.create_material_draw_object_from_static_mesh(
                &find_static_mesh.vertexes,
                &find_static_mesh.indexes,
                Some(format!("{} - {}", &self.name, &find_static_mesh.name)),
                material,
                player_viewport.global_constants_handle.clone(),
                player_viewport.point_lights_constants_handle.clone(),
                player_viewport.spot_lights_constants_handle.clone(),
            );
        } else {
            draw_object = engine.create_draw_object_from_static_mesh(
                &find_static_mesh.vertexes,
                &find_static_mesh.indexes,
                Some(find_static_mesh.name.clone()),
                player_viewport.global_constants_handle.clone(),
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
        let aabb = static_mesh_get_aabb(&find_static_mesh);
        run_time.aabb = Some(aabb);
        run_time.draw_objects = Some(draw_object);
        run_time._mesh = Some(find_static_mesh);
    }

    fn on_is_enable_multiresolution_changed(&mut self) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        let Some(draw_objects) = &mut run_time.draw_objects else {
            return;
        };
        let Some(mesh) = &mut run_time._mesh else {
            return;
        };
        match draw_objects {
            EDrawObjectType::StaticMeshMaterial(draw_objects) => {
                let rm = ResourceManager::default();
                if self.is_enable_multiresolution {
                    draw_objects.multiple_resolution_mesh_pass_resource_handle =
                        rm.get_multiple_resolution_mesh_handle(&mesh.url);
                } else {
                    draw_objects.multiple_resolution_mesh_pass_resource_handle = None;
                }
            }
            _ => {}
        }
    }
}
