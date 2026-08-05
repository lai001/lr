use crate::ui::object_property_view::ObjectPropertyView;
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_core_minimal::types::HasUrl;
use rs_engine::{
    components::{component::Component, text_component::TextComponent},
    content::render_target_2d::RenderTarget2D,
    engine::Engine,
    scene_node::SceneComponent,
};
use rust_i18n::t;
use std::{any::TypeId, borrow::Cow, collections::HashMap};

pub trait ComponentEditable: 'static {
    fn edit_default(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
    ) {
        let _ = engine;
        let _ = content_manager;
        if let Some(new_name) = ObjectPropertyView::edit_name(&component.get_name(), ui) {
            component.set_name(new_name);
        }

        let mut transformation = component.get_transformation();
        ObjectPropertyView::transformation_widget_mut(&mut transformation, ui);
        component.set_transformation(transformation);
        ObjectPropertyView::transformation_widget(&component.get_final_transformation(), ui);
    }

    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
    ) {
        Self::edit_default(self, ui, component, engine, content_manager);
    }

    fn display_type_name(&self) -> Cow<'static, str>;
}

pub struct TextComponentEdit {}

impl ComponentEditable for TextComponentEdit {
    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
    ) {
        Self::edit_default(self, ui, component, engine, content_manager);
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
            let is_changed = super::misc::render_combo_box(
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
    }

    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: TextComponent")
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
