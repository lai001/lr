use crate::ui::component_edit::ComponentEditable;
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{
    collision_componenet::CollisionComponent, components::component::Component, engine::Engine,
};
use rust_i18n::t;
use std::borrow::Cow;

pub struct CollisionComponentEdit {}

impl ComponentEditable for CollisionComponentEdit {
    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
        object_property_view: &crate::ui::object_property_view::ObjectPropertyView,
    ) -> Option<Box<dyn super::UIComponentPropertyEvent>> {
        let _ = object_property_view;
        let _ = content_manager;
        let _ = engine;

        let component = component
            .downcast_mut::<CollisionComponent>()
            .expect("Matched type");
        ui.checkbox(
            &mut component.is_show_preview,
            t!("Is show preview").as_ref(),
        );
        None
    }

    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: CollisionComponent")
    }
}
