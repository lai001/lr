use crate::ui::{component_edit::ComponentEditable, misc};
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_core_minimal::types::HasUrl;
use rs_engine::{
    components::{component::Component, text_component::TextComponent},
    content::render_target_2d::RenderTarget2D,
    engine::Engine,
};
use rust_i18n::t;
use std::borrow::Cow;

pub struct TextComponentEdit {}

impl ComponentEditable for TextComponentEdit {
    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
        object_property_view: &crate::ui::object_property_view::ObjectPropertyView,
    ) -> Option<Box<dyn super::UIComponentPropertyEvent>> {
        let _ = object_property_view;
        let component = component
            .downcast_mut::<TextComponent>()
            .expect("Matched type");
        {
            ui.horizontal(|ui| {
                ui.label(t!("Text: ").as_ref());
                ui.text_edit_singleline(&mut component.text);
            });
        }
        {
            let mut current_url = component.render_target.as_ref();
            let candidate_items = content_manager
                .content_files()
                .iter()
                .map(|rt| {
                    if let Some(rt) = rt.borrow().downcast_ref::<RenderTarget2D>() {
                        return Some(rt.get_url());
                    }
                    return None;
                })
                .flatten()
                .collect::<Vec<url::Url>>();
            let is_changed = misc::render_combo_box(
                ui,
                t!("Render Target"),
                Some(egui::Id::new("Render Target")),
                &mut current_url,
                &candidate_items,
            );
            if is_changed {
                component.set_render_target(
                    current_url.cloned(),
                    engine,
                    &content_manager.content_map(),
                );
            }
        }

        None
    }

    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: TextComponent")
    }
}
