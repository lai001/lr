use crate::content::material::ParamentResource;
#[cfg(feature = "network")]
use crate::network::NetworkReplicated;
#[cfg(feature = "network")]
use crate::network::{self};
use crate::physics_ability::{self};
use crate::uniform_map::UniformMap;
use crate::{
    content::{content_file_type::EContentFileType, level::LevelPhysics, material::Material},
    drawable::EDrawObjectType,
    engine::Engine,
    misc::{static_mesh_get_aabb, transform_aabb},
    player_viewport::PlayerViewport,
    resource_manager::ResourceManager,
};
use rapier3d::prelude::*;
use rs_artifact::material_paramenters::BaseDataValueType;
use rs_artifact::static_mesh::StaticMesh;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::command::EBindingResource;
use rs_render_types::MaterialOptions;
use serde::{Deserialize, Serialize};
#[cfg(feature = "network")]
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AgentTransformation {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
}

impl AgentTransformation {
    pub fn abs_diff_eq(&self, rhs: &Self) -> bool {
        let max_abs_diff = 1.0e-6;
        self.rotation.abs_diff_eq(rhs.rotation, max_abs_diff)
            && self.translation.abs_diff_eq(rhs.translation, max_abs_diff)
    }
}

#[derive(Clone)]
pub struct Physics {
    pub colliders: Vec<Collider>,
    pub collider_handles: Vec<ColliderHandle>,
    pub rigid_body: RigidBody,
    pub rigid_body_handle: RigidBodyHandle,
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
    parament_resource: Option<crate::content::material::ParamentResource>,
    is_parament_resource_dirty: bool,
    _mesh: Option<Arc<StaticMesh>>,
    pub physics: Option<physics_ability::PhysicsAbility>,
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    aabb: Option<Aabb>,
    pending_rigid_body: Option<RigidBody>,
    pending_agent_transformation: Option<AgentTransformation>,
}

#[cfg(feature = "network")]
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum ReplicatedFieldType {
    IsVisible,
    Transformation,
    PhysicsRigidBody,
    AgentTransformation,
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
    #[serde(skip)]
    is_sync_with_server: bool,
    #[serde(skip)]
    net_mode: network::ENetMode,
    #[serde(skip)]
    debug_description: Option<String>,
    #[serde(skip)]
    agent_transformation: Option<AgentTransformation>,
}

#[cfg(feature = "network")]
impl NetworkFields {
    pub fn new() -> NetworkFields {
        NetworkFields {
            net_id: Some(network::default_uuid()),
            is_replicated: false,
            replicated_datas: TransmissionType::new(),
            is_sync_with_server: false,
            net_mode: network::ENetMode::Server,
            debug_description: None,
            agent_transformation: None,
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
    pub physics: physics_ability::Initialization,
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

        let replicated_field_types = self
            .network_fields
            .replicated_datas
            .keys()
            .map(|x| format!("{:?}", x))
            .collect::<Vec<String>>()
            .join(", ");
        let description = format!(
            "Name: {}, Field types: {}",
            &self.name, &replicated_field_types
        );
        self.network_fields.debug_description = Some(description);
        self.network_fields.reset();
        let data = encoded_data.unwrap_or_default();
        data
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
                    ReplicatedFieldType::PhysicsRigidBody => {
                        //
                        match self.network_fields.net_mode {
                            network::ENetMode::Server => {
                                let rigid_body: RigidBody =
                                    rs_artifact::bincode_legacy::deserialize(&v, None)?;
                                let Some(run_time) = self.run_time.as_mut() else {
                                    panic!();
                                };
                                run_time.pending_rigid_body = Some(rigid_body);
                            }
                            network::ENetMode::Client => {}
                        }
                    }
                    ReplicatedFieldType::AgentTransformation => {
                        let agent_transformation: AgentTransformation =
                            rs_artifact::bincode_legacy::deserialize(&v, None)?;
                        match self.network_fields.net_mode {
                            network::ENetMode::Server => {}
                            network::ENetMode::Client => {
                                let Some(run_time) = self.run_time.as_mut() else {
                                    panic!();
                                };
                                run_time.pending_agent_transformation = Some(agent_transformation);
                            }
                        }
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

    fn debug_description(&self) -> Option<String> {
        self.network_fields.debug_description.clone()
    }

    fn sync_with_server(&mut self, is_sync: bool) {
        self.network_fields.is_sync_with_server = is_sync;
    }

    fn is_sync_with_server(&self) -> bool {
        self.network_fields.is_sync_with_server
    }

    fn on_net_mode_changed(&mut self, net_mode: network::ENetMode) {
        self.network_fields.net_mode = net_mode;
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
        let shape_type = physics_ability::EShapeType::Mesh(physics_ability::MeshOptions {
            mesh_url: static_mesh_url.clone(),
            is_use_convex_decomposition: false,
        });
        StaticMeshComponent {
            name,
            transformation,
            material_url,
            run_time: None,
            static_mesh: static_mesh_url,
            is_visible: true,
            physics: physics_ability::Initialization {
                rigid_body_type: RigidBodyType::Dynamic,
                shape_type,
            },
            is_enable_multiresolution: false,
            #[cfg(feature = "network")]
            network_fields: NetworkFields::new(),
        }
    }

    pub fn new_sp(
        name: String,
        static_mesh_url: Option<url::Url>,
        material_url: Option<url::Url>,
        transformation: glam::Mat4,
    ) -> SingleThreadMutType<StaticMeshComponent> {
        SingleThreadMut::new(Self::new(
            name,
            static_mesh_url,
            material_url,
            transformation,
        ))
    }

    pub fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        assert!(self.run_time.is_none());
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
                    if find_static_mesh.is_some() {
                        break;
                    }
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

        if let Some(url) = &self.static_mesh {
            if find_static_mesh.is_none() {
                log::warn!("Can not find static mesh {}", url);
            }
        }

        if let Some(url) = &self.material_url {
            if material.is_none() {
                log::warn!("Can not find material {}", url);
            }
        }
        let parament_resource = Self::create_parament_resource(engine, material.clone());

        let (draw_object, mesh, aabb) = if let Some(find_static_mesh) = find_static_mesh {
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
                    if let Some(parament_resource) = &parament_resource {
                        draw_object.user_paramenters.push(
                            rs_render::command::EBindingResource::Constants(
                                *parament_resource.handle,
                            ),
                        );
                    }
                }
                _ => unimplemented!(),
            }
            let aabb = static_mesh_get_aabb(&find_static_mesh);
            (Some(draw_object), Some(find_static_mesh), Some(aabb))
        } else {
            (None, None, None)
        };
        self.run_time = Some(StaticMeshComponentRuntime {
            draw_objects: draw_object,
            _mesh: mesh,
            physics: None,
            parent_final_transformation: glam::Mat4::IDENTITY,
            final_transformation: glam::Mat4::IDENTITY,
            aabb,
            pending_rigid_body: None,
            pending_agent_transformation: None,
            parament_resource: parament_resource,
            is_parament_resource_dirty: false,
        });
        self.on_is_enable_multiresolution_changed();
    }

    fn create_parament_resource(
        engine: &mut Engine,
        material: Option<std::rc::Rc<std::cell::RefCell<Material>>>,
    ) -> Option<ParamentResource> {
        let mut parament_resource: Option<ParamentResource> = None;
        if let Some(material) = material.clone() {
            let material = material.borrow();
            let material_info = material.get_material_info();
            let material_info = material_info
                .get(&MaterialOptions { is_skin: false })
                .expect("Valid");
            assert!(material_info.paramenters.len() <= 1);
            for parament in &material_info.paramenters {
                if parament.is_valid() {
                    let uniform_map = UniformMap::new(&parament.fields);
                    let buffer_handle = engine.create_buffer(
                        uniform_map.get_data().to_vec(),
                        wgpu::BufferUsages::UNIFORM,
                        None,
                    );
                    if let Ok(buffer_handle) = buffer_handle {
                        parament_resource = Some(ParamentResource {
                            handle: buffer_handle,
                            uniform_map: uniform_map,
                        });
                    }
                }
            }
        }
        parament_resource
    }

    pub fn set_material_value(&mut self, name: &str, value: BaseDataValueType) -> bool {
        let Some(run_time) = &mut self.run_time else {
            return false;
        };
        let Some(parament_resource) = run_time.parament_resource.as_mut() else {
            return false;
        };
        let is_success = match value {
            BaseDataValueType::F32(value) => parament_resource
                .uniform_map
                .set_field_f32_value(name, value),
            BaseDataValueType::Vec2(value) => parament_resource
                .uniform_map
                .set_field_vec2_value(name, value),
            BaseDataValueType::Vec3(value) => parament_resource
                .uniform_map
                .set_field_vec3_value(name, value),
            BaseDataValueType::Vec4(value) => parament_resource
                .uniform_map
                .set_field_vec4_value(name, value),
        };
        if is_success {
            run_time.is_parament_resource_dirty = true;
        }
        return is_success;
    }

    pub fn tick(&mut self, time: f32, engine: &mut Engine, level_physics: &mut LevelPhysics) {
        let _ = time;
        let _ = engine;
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        let Some(mut draw_objects) = run_time.draw_objects.as_mut() else {
            return;
        };

        if run_time.is_parament_resource_dirty {
            run_time.is_parament_resource_dirty = false;
            if let Some(parament_resource) = run_time.parament_resource.as_mut() {
                let buffer_handle = engine
                    .create_buffer(
                        parament_resource.uniform_map.get_data().to_vec(),
                        wgpu::BufferUsages::UNIFORM,
                        None,
                    )
                    .expect("Valid");
                parament_resource.handle = buffer_handle.clone();
                match &mut draw_objects {
                    EDrawObjectType::StaticMeshMaterial(draw_object) => {
                        draw_object.user_paramenters =
                            vec![EBindingResource::Constants(*buffer_handle)];
                    }
                    _ => {}
                }
            };
        }

        if let Some(physics) = run_time.physics.as_mut() {
            if let Some(pending_rigid_body) = run_time.pending_rigid_body.take() {
                let handle = level_physics.rigid_body_set.insert(pending_rigid_body);
                physics.collider_handles.clear();
                for collider in physics.colliders.clone() {
                    let collider_handle = level_physics.collider_set.insert_with_parent(
                        collider,
                        handle,
                        &mut level_physics.rigid_body_set,
                    );
                    physics.collider_handles.push(collider_handle);
                }
                level_physics.remove_rigid_body(physics.rigid_body_handle);
                physics.rigid_body_handle = handle;
            }
        }

        let is_simulate = run_time
            .physics
            .as_mut()
            .map(|x| x.is_apply_simulate)
            .unwrap_or(false);

        #[cfg(feature = "network")]
        let mut send_agent_transformation: Option<AgentTransformation> = None;
        match (run_time.physics.as_mut(), is_simulate) {
            (Some(physics), true) => {
                let transformation = if let Some(AgentTransformation {
                    translation,
                    rotation,
                }) = &run_time.pending_agent_transformation
                {
                    let scale = run_time
                        .final_transformation
                        .to_scale_rotation_translation()
                        .0;
                    glam::Mat4::from_scale_rotation_translation(scale, *rotation, *translation)
                } else {
                    let rigid_body = &level_physics.rigid_body_set[physics.rigid_body_handle];
                    let translation = rigid_body.translation();
                    let translation = glam::vec3(translation.x, translation.y, translation.z);
                    let rotation = rigid_body.rotation();
                    let scale = run_time
                        .final_transformation
                        .to_scale_rotation_translation()
                        .0;
                    #[cfg(feature = "network")]
                    match &self.network_fields.net_mode {
                        network::ENetMode::Server => {
                            send_agent_transformation = Some(AgentTransformation {
                                translation: translation,
                                rotation: *rotation,
                            });
                        }
                        network::ENetMode::Client => {}
                    }
                    glam::Mat4::from_scale_rotation_translation(scale, *rotation, translation)
                };

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
        #[cfg(feature = "network")]
        if let Some(send_agent_transformation) = send_agent_transformation {
            let _ = self.set_agent_transformation(send_agent_transformation);
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

    pub fn initialize_physics(
        &mut self,
        engine: &mut Engine,
        level_physics: &mut LevelPhysics,
        files: &[EContentFileType],
    ) {
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        if run_time.physics.is_some() {
            log::warn!("Double initialize physics, {}", self.name);
        }
        let resource_manager = engine.get_resource_manager().clone();
        let physics = physics_ability::PhysicsAbility::new(
            &self.physics,
            run_time.final_transformation,
            true,
            files,
            resource_manager,
            level_physics,
        );
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
        engine: &mut Engine,
        level_physics: Option<&mut LevelPhysics>,
        files: &[EContentFileType],
    ) {
        let _ = engine;
        let _ = files;
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };

        let Some(physics) = run_time.physics.as_mut() else {
            return;
        };
        let Some(level_physics) = level_physics else {
            return;
        };
        if !physics.is_valid() {
            return;
        }

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
        rigid_body.set_translation(translation, false);
        let (axis, angle) = rotation.to_axis_angle();
        rigid_body.set_rotation(Rotation::from_axis_angle(axis.normalize(), angle), false);
        rigid_body.set_angvel(glam::Vec3::ZERO, false);
        rigid_body.set_linvel(glam::Vec3::ZERO, false);
        rigid_body.reset_forces(false);
        rigid_body.reset_torques(false);
        rigid_body.wake_up(true);

        collider.set_position(Pose3::from_translation(translation));
        collider.set_rotation(Rotation::from_axis_angle(axis.normalize(), angle));
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut physics_ability::PhysicsAbility> {
        self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten()
    }

    pub fn get_physics(&self) -> Option<&physics_ability::PhysicsAbility> {
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

    #[cfg(feature = "network")]
    fn sync_physics(&mut self) -> rs_artifact::error::Result<()> {
        let Some(run_time) = self.run_time.as_mut() else {
            panic!();
        };
        match self.network_fields.net_mode {
            network::ENetMode::Server => {}
            network::ENetMode::Client => {
                if let Some(physics) = run_time.physics.as_mut() {
                    let data = rs_artifact::bincode_legacy::serialize(&physics.rigid_body, None)?;
                    self.network_fields
                        .replicated_datas
                        .insert(ReplicatedFieldType::PhysicsRigidBody, data);
                }
            }
        }
        Ok(())
    }

    pub fn modify_physics(
        &mut self,
        level_physics: &mut LevelPhysics,
        mut modify: impl FnMut(&mut RigidBody) -> (),
    ) -> rs_artifact::error::Result<()> {
        let Some(run_time) = self.run_time.as_mut() else {
            panic!();
        };
        if let Some(physics) = run_time.physics.as_mut() {
            if physics.rigid_body_handle != RigidBodyHandle::invalid() {
                modify(&mut physics.rigid_body);
                let exist = &mut level_physics.rigid_body_set[physics.rigid_body_handle];
                exist.copy_from(&physics.rigid_body);
                #[cfg(feature = "network")]
                return self.sync_physics();
            }
        }
        return Ok(());
    }

    #[cfg(feature = "network")]
    pub fn set_agent_transformation(
        &mut self,
        agent_transformation: AgentTransformation,
    ) -> rs_artifact::error::Result<()> {
        if let Some(lhs) = self.network_fields.agent_transformation.as_ref() {
            if lhs.abs_diff_eq(&agent_transformation) {
                return Ok(());
            }
        }
        let data = rs_artifact::bincode_legacy::serialize(&agent_transformation, None)?;
        self.network_fields.agent_transformation = Some(agent_transformation);
        self.network_fields
            .replicated_datas
            .insert(ReplicatedFieldType::AgentTransformation, data);
        Ok(())
    }
}
