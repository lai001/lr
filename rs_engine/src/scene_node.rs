#[cfg(feature = "network")]
use crate::network;
#[cfg(feature = "network")]
use crate::network::NetworkReplicated;
use crate::{
    components::component::Component,
    content::{content_file_type::EContentFileType, level::LevelPhysics},
    engine::Engine,
    player_viewport::PlayerViewport,
    static_mesh_component::StaticMeshComponent,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

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

#[typetag::serde]
impl Component for SceneComponent {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    fn get_final_transformation(&self) -> glam::Mat4 {
        let final_transformation = self
            .run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default();
        final_transformation
    }

    fn set_transformation(&mut self, transformation: glam::Mat4) {
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> glam::Mat4 {
        self.transformation
    }

    fn on_post_update_transformation(
        &mut self,
        engine: &mut Engine,
        level_physics: Option<&mut LevelPhysics>,
        files: &[EContentFileType],
    ) {
        let _ = files;
        let _ = engine;
        let _ = level_physics;
    }

    fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    fn initialize(
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

    fn initialize_physics(
        &mut self,
        engine: &mut Engine,
        level_physics: &mut LevelPhysics,
        files: &[EContentFileType],
    ) {
        let _ = files;
        let _ = engine;
        let _ = level_physics;
    }

    fn tick(&mut self, time: f32, engine: &mut Engine, level_physics: &mut LevelPhysics) {
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

    #[cfg(feature = "network")]
    fn as_network_replicated_mut(&mut self) -> Option<&mut dyn crate::network::NetworkReplicated> {
        Some(self)
    }

    #[cfg(feature = "network")]
    fn as_network_replicated(&self) -> Option<&dyn crate::network::NetworkReplicated> {
        Some(self)
    }
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

    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_draw_objects(&self) -> Vec<&crate::drawable::EDrawObjectType> {
        vec![]
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
pub struct SceneNode {
    component: SingleThreadMutType<Box<dyn Component>>,
    childs: Vec<SingleThreadMutType<SceneNode>>,
}

impl SceneNode {
    pub fn component(&self) -> std::cell::Ref<'_, Box<dyn Component>> {
        let refe = self.component.borrow();
        refe
    }

    pub fn component_mut(&mut self) -> std::cell::RefMut<'_, Box<dyn Component>> {
        let refe = self.component.borrow_mut();
        refe
    }

    pub fn typed_component<T: Component>(&self) -> Option<std::cell::Ref<'_, T>> {
        std::cell::Ref::filter_map(self.component.borrow(), |component| {
            component.downcast_ref::<T>()
        })
        .ok()
    }

    pub fn is_typed_component<T: Component>(&self) -> bool {
        self.typed_component::<T>().is_some()
    }

    pub fn typed_component_mut<T: Component>(&mut self) -> Option<std::cell::RefMut<'_, T>> {
        std::cell::RefMut::filter_map(self.component.borrow_mut(), |component| {
            component.downcast_mut::<T>()
        })
        .ok()
    }

    pub fn childs(&self) -> &[SingleThreadMutType<SceneNode>] {
        &self.childs
    }

    pub fn underlying_component(&self) -> SingleThreadMutType<Box<dyn Component>> {
        self.component.clone()
    }

    pub fn set_childs(
        &mut self,
        new_childs: Vec<SingleThreadMutType<SceneNode>>,
    ) -> Vec<SingleThreadMutType<SceneNode>> {
        let old = self.childs.clone();
        self.childs = new_childs;
        old
    }

    pub fn add_child(&mut self, child: SingleThreadMutType<SceneNode>) -> bool {
        if self.childs.iter().find(|x| Rc::ptr_eq(x, &child)).is_some() {
            return false;
        }
        self.childs.push(child);
        return true;
    }
}

impl SceneNode {
    pub fn new(name: String) -> SceneNode {
        let component = SingleThreadMut::new(Box::new(SceneComponent::new(
            name,
            glam::Mat4::IDENTITY,
        )) as Box<dyn Component>);
        SceneNode {
            component,
            childs: vec![],
        }
    }

    pub fn new_sp(name: String) -> SingleThreadMutType<SceneNode> {
        SingleThreadMut::new(Self::new(name))
    }

    pub fn from_component(component: impl Component) -> SceneNode {
        SceneNode {
            component: SingleThreadMut::new(Box::new(component)),
            childs: vec![],
        }
    }

    pub fn from_component_box(component: Box<dyn Component>) -> SceneNode {
        SceneNode {
            component: SingleThreadMut::new(component),
            childs: vec![],
        }
    }

    pub fn get_aabb(&self) -> Option<rapier3d::prelude::Aabb> {
        self.typed_component::<StaticMeshComponent>()
            .map(|component| component.get_aabb().clone())
            .flatten()
    }

    pub fn notify_transformation_updated(
        &mut self,
        engine: &mut Engine,
        mut level_physics: Option<&mut LevelPhysics>,
        files: &[EContentFileType],
    ) {
        let parent_transformation = {
            let mut this_component = self.component_mut();
            let parent_final_transformation = this_component.get_parent_final_transformation();
            let final_transformation =
                parent_final_transformation * this_component.get_transformation();
            this_component.set_final_transformation(final_transformation);

            if let Some(level_physics) = level_physics.as_mut() {
                this_component.on_post_update_transformation(engine, Some(level_physics), files);
            } else {
                this_component.on_post_update_transformation(engine, None, files);
            }
            let parent_transformation = this_component.get_final_transformation();
            parent_transformation
        };

        for child in self.childs.clone() {
            crate::actor::Actor::set_world_transformation_recursion(
                &mut child.borrow_mut(),
                parent_transformation,
            );
        }
        if let Some(level_physics) = level_physics.as_mut() {
            for child in self.childs.clone() {
                crate::actor::Actor::on_post_update_transformation_recursion(
                    &mut child.borrow_mut(),
                    engine,
                    Some(level_physics),
                    files,
                );
            }
        } else {
            for child in self.childs.clone() {
                crate::actor::Actor::on_post_update_transformation_recursion(
                    &mut child.borrow_mut(),
                    engine,
                    None,
                    files,
                );
            }
        }
    }

    pub fn changed_state(&self) -> Option<ChangedStateFlags> {
        self.typed_component::<SceneComponent>()
            .map(|component| component.changed_state())
            .flatten()
    }

    pub fn insert_changed_state(&mut self, state: ChangedStateFlags) {
        if let Some(mut component) = self.typed_component_mut::<SceneComponent>() {
            component.insert_changed_state(state);
        }
    }

    pub fn set_changed_state(&mut self, state: ChangedStateFlags) {
        if let Some(mut component) = self.typed_component_mut::<SceneComponent>() {
            component.set_changed_state(state);
        }
    }
}
