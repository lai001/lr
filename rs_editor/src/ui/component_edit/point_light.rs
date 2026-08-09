use crate::ui::{component_edit::ComponentEditable, object_property_view::ObjectPropertyView};
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{
    components::{component::Component, point_light_component::PointLightComponent},
    engine::Engine,
};
use rust_i18n::t;
use std::borrow::Cow;

pub struct PointLightComponentEdit {}

impl ComponentEditable for PointLightComponentEdit {
    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
        object_property_view: &ObjectPropertyView,
    ) -> Option<Box<dyn super::UIComponentPropertyEvent>> {
        let _ = object_property_view;
        let _ = content_manager;
        let _ = engine;

        let component = component
            .downcast_mut::<PointLightComponent>()
            .expect("Matched type");

        ui.vertical(|ui| {
            ObjectPropertyView::vec3_widget_mut(
                &mut component.point_light.ambient,
                ui,
                t!("Ambient"),
                true,
            );
            ObjectPropertyView::vec3_widget_mut(
                &mut component.point_light.diffuse,
                ui,
                t!("Diffuse"),
                true,
            );
            ObjectPropertyView::vec3_widget_mut(
                &mut component.point_light.specular,
                ui,
                t!("Specular"),
                true,
            );
            ui.add(
                egui::DragValue::new(&mut component.point_light.constant)
                    .speed(0.1)
                    .prefix(t!("Constant: ").as_ref()),
            );
            ui.add(
                egui::DragValue::new(&mut component.point_light.linear)
                    .speed(0.1)
                    .prefix(t!("Linear: ").as_ref()),
            );
            ui.add(
                egui::DragValue::new(&mut component.point_light.quadratic)
                    .speed(0.1)
                    .prefix(t!("Quadratic: ").as_ref()),
            );
        });

        None
    }

    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: PointLightComponent")
    }
}
