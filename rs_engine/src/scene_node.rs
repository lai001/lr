#[cfg(feature = "network")]
use crate::network;
#[cfg(feature = "network")]
use crate::network::NetworkReplicated;
use crate::{
    camera_component::CameraComponent,
    collision_componenet::CollisionComponent,
    components::{
        component::Component, point_light_component::PointLightComponent,
        spot_light_component::SpotLightComponent,
    },
    content::{content_file_type::EContentFileType, level::LevelPhysics},
    engine::Engine,
    player_viewport::PlayerViewport,
    skeleton_mesh_component::SkeletonMeshComponent,
    static_mesh_component::StaticMeshComponent,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Clone)]
    pub struct ChangedStateFlags: u8 {
        const Transformation = 1;
    }
}

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum ReplicatedFieldType {
    Transformation,
}

#[cfg(feature = "network")]
type TransmissionType = std::collections::HashMap<ReplicatedFieldType, Vec<u8>>;

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NetworkFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) net_id: Option<uuid::Uuid>,
    #[serde(default = "bool::default")]
    pub is_replicated: bool,
    #[serde(skip)]
    replicated_datas: TransmissionType,
    #[serde(skip)]
    is_sync_with_server: bool,
    #[serde(skip)]
    net_mode: network::ENetMode,
}

#[cfg(feature = "network")]
impl NetworkFields {
    pub fn new() -> NetworkFields {
        NetworkFields {
            net_id: Some(crate::network::default_uuid()),
            is_replicated: false,
            replicated_datas: TransmissionType::new(),
            is_sync_with_server: false,
            net_mode: network::ENetMode::Server,
        }
    }

    pub fn reset(&mut self) {
        self.replicated_datas.drain();
    }
}

#[derive(Clone)]
struct SceneComponentRuntime {
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    net_transformation: Option<glam::Mat4>,
    changed_state: ChangedStateFlags,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    #[serde(skip)]
    run_time: Option<SceneComponentRuntime>,
    #[cfg(feature = "network")]
    #[serde(default)]
    network_fields: NetworkFields,
}

#[cfg(feature = "network")]
impl SceneComponent {
    pub fn network_set_transformation(
        &mut self,
        transformation: glam::Mat4,
    ) -> rs_artifact::error::Result<()> {
        let is_same = self.transformation == transformation;
        if is_same {
            return Ok(());
        }
        self.transformation = transformation;
        self.insert_changed_state(ChangedStateFlags::Transformation);
        let data = rs_artifact::bincode_legacy::serialize(&transformation, None)?;
        self.network_fields
            .replicated_datas
            .insert(ReplicatedFieldType::Transformation, data);
        Ok(())
    }
}

#[cfg(feature = "network")]
impl crate::network::NetworkReplicated for SceneComponent {
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

    fn sync_with_server(&mut self, is_sync: bool) {
        self.network_fields.is_sync_with_server = is_sync;
    }

    fn is_sync_with_server(&self) -> bool {
        self.network_fields.is_sync_with_server
    }

    fn debug_name(&self) -> Option<String> {
        Some(self.name.clone())
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
                        let transformation =
                            rs_artifact::bincode_legacy::deserialize::<glam::Mat4>(&v, None)?;
                        if let Some(runtime) = self.run_time.as_mut() {
                            runtime.net_transformation = Some(transformation);
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

    fn on_net_mode_changed(&mut self, net_mode: network::ENetMode) {
        self.network_fields.net_mode = net_mode;
    }
}

impl SceneComponent {
    pub fn new(name: String, transformation: glam::Mat4) -> SceneComponent {
        SceneComponent {
            name,
            transformation,
            run_time: Some(SceneComponentRuntime {
                final_transformation: glam::Mat4::IDENTITY,
                parent_final_transformation: glam::Mat4::IDENTITY,
                net_transformation: None,
                changed_state: ChangedStateFlags::empty(),
            }),
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
            self.set_network_id(network::default_uuid());
        }
        let _ = player_viewport;
        let _ = files;
        let _ = engine;
        self.run_time = Some(SceneComponentRuntime {
            final_transformation: glam::Mat4::IDENTITY,
            parent_final_transformation: glam::Mat4::IDENTITY,
            net_transformation: None,
            changed_state: ChangedStateFlags::empty(),
        });
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

    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_transformation(&self) -> &glam::Mat4 {
        &self.transformation
    }

    pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    pub fn get_final_transformation(&self) -> glam::Mat4 {
        let final_transformation = self
            .run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default();
        final_transformation
    }

    pub fn on_post_update_transformation(&mut self, level_physics: Option<&mut LevelPhysics>) {
        let _ = level_physics;
    }

    pub fn get_draw_objects(&self) -> Vec<&crate::drawable::EDrawObjectType> {
        vec![]
    }

    pub fn initialize_physics(&mut self, level_physics: &mut LevelPhysics) {
        let _ = level_physics;
    }

    pub fn tick(&mut self, time: f32, engine: &mut Engine, level_physics: &mut LevelPhysics) {
        let _ = engine;
        let _ = time;
        let _ = level_physics;
        if let Some(run_time) = self.run_time.as_mut() {
            if let Some(transformation) = run_time.net_transformation.take() {
                self.transformation = transformation;
                self.insert_changed_state(ChangedStateFlags::Transformation);
            }
        }
    }

    pub fn changed_state(&self) -> Option<ChangedStateFlags> {
        self.run_time.as_ref().map(|x| x.changed_state.clone())
    }

    pub fn insert_changed_state(&mut self, state: ChangedStateFlags) {
        if let Some(runtime) = &mut self.run_time {
            runtime.changed_state.insert(state);
        }
    }

    pub fn set_changed_state(&mut self, state: ChangedStateFlags) {
        if let Some(runtime) = &mut self.run_time {
            runtime.changed_state = state;
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EComponentType {
    SceneComponent(SingleThreadMutType<SceneComponent>),
    StaticMeshComponent(SingleThreadMutType<StaticMeshComponent>),
    SkeletonMeshComponent(SingleThreadMutType<SkeletonMeshComponent>),
    CameraComponent(SingleThreadMutType<CameraComponent>),
    CollisionComponent(SingleThreadMutType<CollisionComponent>),
    SpotLightComponent(SingleThreadMutType<SpotLightComponent>),
    PointLightComponent(SingleThreadMutType<PointLightComponent>),
}

macro_rules! copy_fn {
    ($($x:tt),*) => {
        pub fn copy(&self) -> EComponentType {
            match self {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        let copy_component = component.clone();
                        EComponentType::$x(SingleThreadMut::new(copy_component))
                    }
                )*
            }
        }
    }
}

impl EComponentType {
    copy_fn!(
        SceneComponent,
        StaticMeshComponent,
        SkeletonMeshComponent,
        CameraComponent,
        CollisionComponent,
        SpotLightComponent,
        PointLightComponent
    );
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneNode {
    pub component: EComponentType,
    pub childs: Vec<SingleThreadMutType<SceneNode>>,
}

macro_rules! common_fn {
    ($($x:tt),*) => {
        pub fn get_name(&self) -> String {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.name.clone()
                    }
                )*
            }
        }

        pub fn set_name(&self, new_name: String) {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.name = new_name;
                    }
                )*
            }
        }

        pub fn get_final_transformation(&self) -> glam::Mat4 {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.get_final_transformation()
                    }
                )*
            }
        }


        pub fn set_transformation(&mut self, transformation: glam::Mat4) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.transformation = transformation;
                    }
                )*
            }
        }

        pub fn get_transformation(&self) -> glam::Mat4 {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.transformation
                    }
                )*
            }
        }

        pub fn on_post_update_transformation(&mut self, level_physics: Option<&mut LevelPhysics>) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.on_post_update_transformation(level_physics);
                    }
                )*
            }
        }

        pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.set_final_transformation(final_transformation);
                    }
                )*
            }
        }

        pub fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.set_parent_final_transformation(parent_final_transformation);
                    }
                )*
            }
        }

        pub fn get_parent_final_transformation(&self) -> glam::Mat4 {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.get_parent_final_transformation()
                    }
                )*
            }
        }

        pub fn initialize(&mut self,
            engine: &mut Engine,
            files: &[EContentFileType],
            player_viewport: &mut PlayerViewport,
        ) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.initialize(engine, files, player_viewport);
                    }
                )*
            }
        }

        pub fn initialize_physics(
            &mut self,
            level_physics: &mut LevelPhysics,
        ) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.initialize_physics(level_physics);
                    }
                )*
            }
        }

        pub fn tick(
            &mut self,
            time: f32,
            engine: &mut Engine,
            level_physics: &mut LevelPhysics,
        ) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.tick(time, engine, level_physics);
                    }
                )*
            }
        }
    };
}

impl SceneNode {
    pub fn new(name: String) -> SceneNode {
        SceneNode {
            component: EComponentType::SceneComponent(SingleThreadMut::new(SceneComponent::new(
                name,
                glam::Mat4::IDENTITY,
            ))),
            childs: vec![],
        }
    }

    pub fn new_sp(name: String) -> SingleThreadMutType<SceneNode> {
        SingleThreadMut::new(Self::new(name))
    }

    pub fn static_mesh_component_node(
        component: SingleThreadMutType<StaticMeshComponent>,
    ) -> SingleThreadMutType<SceneNode> {
        SingleThreadMut::new(SceneNode {
            component: EComponentType::StaticMeshComponent(component),
            childs: vec![],
        })
    }

    pub fn get_aabb(&self) -> Option<rapier3d::prelude::Aabb> {
        match &self.component {
            EComponentType::SceneComponent(_) => None,
            EComponentType::StaticMeshComponent(component) => component.borrow().get_aabb(),
            EComponentType::SkeletonMeshComponent(_) => None,
            EComponentType::CameraComponent(_) => None,
            EComponentType::CollisionComponent(_) => None,
            EComponentType::SpotLightComponent(_) => None,
            EComponentType::PointLightComponent(_) => None,
        }
    }

    pub fn notify_transformation_updated(&mut self, mut level_physics: Option<&mut LevelPhysics>) {
        let parent_final_transformation = self.get_parent_final_transformation();
        let final_transformation = parent_final_transformation * self.get_transformation();
        self.set_final_transformation(final_transformation);

        if let Some(level_physics) = level_physics.as_mut() {
            self.on_post_update_transformation(Some(level_physics));
        } else {
            self.on_post_update_transformation(None);
        }
        for child in self.childs.clone() {
            let parent_transformation = self.get_final_transformation();
            crate::actor::Actor::set_world_transformation_recursion(
                &mut child.borrow_mut(),
                parent_transformation,
            );
        }
        if let Some(level_physics) = level_physics.as_mut() {
            for child in self.childs.clone() {
                crate::actor::Actor::on_post_update_transformation_recursion(
                    &mut child.borrow_mut(),
                    Some(level_physics),
                );
            }
        } else {
            for child in self.childs.clone() {
                crate::actor::Actor::on_post_update_transformation_recursion(
                    &mut child.borrow_mut(),
                    None,
                );
            }
        }
    }

    pub fn changed_state(&self) -> Option<ChangedStateFlags> {
        match &self.component {
            EComponentType::SceneComponent(component) => component.borrow().changed_state(),
            _ => None,
        }
    }

    pub fn insert_changed_state(&mut self, state: ChangedStateFlags) {
        match &mut self.component {
            EComponentType::SceneComponent(component) => {
                component.borrow_mut().insert_changed_state(state);
            }
            _ => {}
        }
    }

    pub fn set_changed_state(&mut self, state: ChangedStateFlags) {
        match &mut self.component {
            EComponentType::SceneComponent(component) => {
                component.borrow_mut().set_changed_state(state);
            }
            _ => {}
        }
    }

    common_fn!(
        SceneComponent,
        StaticMeshComponent,
        SkeletonMeshComponent,
        CameraComponent,
        CollisionComponent,
        SpotLightComponent,
        PointLightComponent
    );
}
