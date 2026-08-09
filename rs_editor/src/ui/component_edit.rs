mod camera;
mod collision;
mod point_light;
mod skeleton_mesh;
mod spot_light;
mod static_mesh;
mod text;

use crate::ui::{
    UIEvent,
    component_edit::{
        camera::CameraComponentEdit, collision::CollisionComponentEdit,
        point_light::PointLightComponentEdit, skeleton_mesh::SkeletonMeshComponentEdit,
        spot_light::SpotLightComponentEdit, static_mesh::StaticMeshComponentEdit,
        text::TextComponentEdit,
    },
    object_property_view::ObjectPropertyView,
};
use downcast_rs::impl_downcast;
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{
    camera_component::CameraComponent,
    collision_componenet::CollisionComponent,
    components::{
        component::Component, point_light_component::PointLightComponent,
        spot_light_component::SpotLightComponent, text_component::TextComponent,
    },
    engine::Engine,
    scene_node::SceneComponent,
    skeleton_mesh_component::SkeletonMeshComponent,
    static_mesh_component::StaticMeshComponent,
};
use rust_i18n::t;
use std::{any::TypeId, borrow::Cow, collections::HashMap};

pub trait UIComponentPropertyEvent: UIEvent {}
impl_downcast!(UIComponentPropertyEvent);

pub trait ComponentEditable: 'static {
    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
        object_property_view: &ObjectPropertyView,
    ) -> Option<Box<dyn UIComponentPropertyEvent>> {
        let _ = object_property_view;
        let _ = content_manager;
        let _ = engine;
        let _ = component;
        let _ = ui;
        None
    }

    fn display_type_name(&self) -> Cow<'static, str>;

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        component: &mut dyn Component,
        event: Box<dyn UIComponentPropertyEvent>,
    ) {
        let _ = editor_context;
        let _ = component;
        let _ = event;
    }
}

pub struct SceneComponentEdit {}

impl ComponentEditable for SceneComponentEdit {
    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: SceneComponent")
    }
}

pub struct ComponentEdit {
    editables: HashMap<std::any::TypeId, Box<dyn ComponentEditable>>,
}

impl ComponentEdit {
    pub fn new() -> ComponentEdit {
        let mut editables: HashMap<std::any::TypeId, Box<dyn ComponentEditable>> = HashMap::new();
        let _ = editables.insert(
            TypeId::of::<TextComponent>(),
            Box::new(TextComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<SceneComponent>(),
            Box::new(SceneComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<StaticMeshComponent>(),
            Box::new(StaticMeshComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<CameraComponent>(),
            Box::new(CameraComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<CollisionComponent>(),
            Box::new(CollisionComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<SpotLightComponent>(),
            Box::new(SpotLightComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<PointLightComponent>(),
            Box::new(PointLightComponentEdit {}),
        );
        let _ = editables.insert(
            TypeId::of::<SkeletonMeshComponent>(),
            Box::new(SkeletonMeshComponentEdit {}),
        );
        ComponentEdit { editables }
    }

    pub fn editable(
        &mut self,
        component: &mut dyn Component,
    ) -> Option<&mut Box<dyn ComponentEditable>> {
        let id = component.type_id();
        let editable = self.editables.get_mut(&id);
        editable
    }
}
