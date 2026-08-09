use crate::ui::{component_edit::ComponentEditable, object_property_view::ObjectPropertyView};
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{
    components::{component::Component, spot_light_component::SpotLightComponent},
    engine::Engine,
};
use rust_i18n::t;
use std::borrow::Cow;

pub struct SpotLightComponentEdit {}

impl ComponentEditable for SpotLightComponentEdit {
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
            .downcast_mut::<SpotLightComponent>()
            .expect("Matched type");

        ui.vertical(|ui| {
            ObjectPropertyView::vec3_widget_mut(
                &mut component.spot_light.light.ambient,
                ui,
                t!("Ambient"),
                true,
            );
            ObjectPropertyView::vec3_widget_mut(
                &mut component.spot_light.light.diffuse,
                ui,
                t!("Diffuse"),
                true,
            );
            ObjectPropertyView::vec3_widget_mut(
                &mut component.spot_light.light.specular,
                ui,
                t!("Specular"),
                true,
            );
            ui.add(
                egui::DragValue::new(&mut component.spot_light.light.constant)
                    .speed(0.1)
                    .prefix(t!("Constant: ").as_ref()),
            );
            ui.add(
                egui::DragValue::new(&mut component.spot_light.light.linear)
                    .speed(0.1)
                    .prefix(t!("Linear: ").as_ref()),
            );
            ui.add(
                egui::DragValue::new(&mut component.spot_light.light.quadratic)
                    .speed(0.1)
                    .prefix(t!("Quadratic: ").as_ref()),
            );
            ui.add(
                egui::DragValue::new(&mut component.spot_light.cut_off)
                    .speed(0.1)
                    .prefix(t!("Cut off: ").as_ref()),
            );
            ui.add(
                egui::DragValue::new(&mut component.spot_light.outer_cut_off)
                    .speed(0.1)
                    .prefix(t!("Outer cut off: ").as_ref()),
            );
        });

        None
    }

    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: SpotLightComponent")
    }
}
