use crate::data_source::{DataSource, MeshItem};
use crate::editor_ui::load::ImageLoader;
use crate::ui::content_item_property_view::ContentItemPropertyView;
use crate::ui::gizmo_view::GizmoView;
use crate::ui::material_view::MaterialView;
use crate::ui::top_menu::TopMenu;
use crate::ui::{
    asset_view, console_cmds_view, content_browser, gizmo_settings, level_view, top_menu,
};
use egui::*;
use rs_engine::input_mode::EInputMode;
use std::sync::Arc;
use std::{path::PathBuf, rc::Rc};
use transform_gizmo_egui::GizmoResult;

#[derive(Debug)]
pub struct ClickMeshItem {
    pub file_path: PathBuf,
    pub item: Rc<MeshItem>,
}

#[derive(Default)]
pub struct ClickEvent {
    pub click_actor: Option<level_view::EClickEventType>,
    pub mesh_item: Option<ClickMeshItem>,
    pub click_aseet: Option<asset_view::EClickItemType>,
    pub menu_event: Option<top_menu::EClickEventType>,
    pub content_browser_event: Option<content_browser::EClickEventType>,
    pub gizmo_result: Option<GizmoResult>,
}

pub struct EditorUI {
    image_loader: Option<Arc<dyn ImageLoader + Send + Sync + 'static>>,
    svg_loader: Option<Arc<dyn ImageLoader + Send + Sync + 'static>>,
    asset_folder_path: Option<PathBuf>,
    top_menu: TopMenu,
    gizmo_view: GizmoView,
    pub material_view: MaterialView,
    pub egui_context: Context,
    pub content_item_property_view: ContentItemPropertyView,
}

impl EditorUI {
    pub fn new(context: &Context) -> Self {
        let image_loader_id = "egui_extras::loaders::image_loader::ImageCrateLoader";
        let svg_loader_id = "egui_extras::loaders::svg_loader::SvgLoader";
        let mut image_loader = None;
        let mut svg_loader = None;
        egui_extras::install_image_loaders(context);
        for item in context.loaders().image.lock().iter() {
            if item.id() == image_loader_id {
                image_loader = Some(item.clone());
            }
            if item.id() == svg_loader_id {
                svg_loader = Some(item.clone());
            }
        }
        Self {
            image_loader,
            svg_loader,
            asset_folder_path: None,
            top_menu: TopMenu {
                new_project_name: String::new(),
            },
            gizmo_view: GizmoView::default(),
            material_view: MaterialView::new(),
            egui_context: context.clone(),
            content_item_property_view: ContentItemPropertyView::new(),
        }
    }

    pub fn set_asset_folder_path(&mut self, asset_folder_path: Option<PathBuf>) {
        self.asset_folder_path = asset_folder_path;
    }

    pub fn build(&mut self, context: &Context, data_source: &mut DataSource) -> ClickEvent {
        let mut click = ClickEvent::default();
        click.menu_event = self.top_menu.draw(context, data_source);

        Self::model_hierarchy_window(context, data_source, &mut click);
        if let Some(level) = &data_source.level {
            let window = Self::new_window("Level", data_source.input_mode);
            click.click_actor = crate::ui::level_view::draw(
                window,
                context,
                &mut data_source.is_level_view_open,
                &level.as_ref().borrow(),
            );
        }
        let window = Self::new_window("Asset", data_source.input_mode);
        click.click_aseet = asset_view::draw(
            window,
            context,
            &mut data_source.is_asset_folder_open,
            data_source.current_asset_folder.as_ref(),
            data_source.highlight_asset_file.as_ref(),
        );

        let window = Self::new_window("Gizmo Settings", data_source.input_mode);
        gizmo_settings::draw(
            window,
            context,
            &mut self.gizmo_view.visuals,
            &mut self.gizmo_view.gizmo_mode,
            &mut self.gizmo_view.gizmo_orientation,
            &mut self.gizmo_view.custom_highlight_color,
        );
        if let Some(project_settings) = data_source.project_settings.clone() {
            let window = Self::new_window("Project Settings", data_source.input_mode);
            crate::ui::project_settings::draw(
                window,
                context,
                &mut data_source.project_settings_open,
                project_settings,
            );
        }
        if let Some(asset_folder_path) = self.asset_folder_path.as_ref() {
            let window = Self::new_window("Content Browser", data_source.input_mode);
            click.content_browser_event = content_browser::draw(
                window,
                context,
                asset_folder_path,
                &mut data_source.content_data_source,
                // data_source.input_mode,
            );
        }
        if let Some(console_cmds) = &data_source.console_cmds {
            let window = Self::new_window("Console Cmds", data_source.input_mode);
            console_cmds_view::draw(
                window,
                context,
                &mut data_source.is_console_cmds_view_open,
                &mut console_cmds.borrow_mut(),
            );
        }

        Self::new_window("Property", data_source.input_mode)
            .open(&mut data_source.is_content_item_property_view_open)
            .vscroll(true)
            .hscroll(true)
            .resizable(true)
            .default_size([250.0, 500.0])
            .show(context, |ui| {
                self.content_item_property_view.draw(ui);
            });

        click
    }

    pub fn draw_material_view(&mut self, context: &Context, data_source: &mut DataSource) {
        self.material_view
            .draw(data_source.current_open_material.clone(), context);
    }

    fn model_hierarchy_window(
        context: &Context,
        data_source: &mut DataSource,
        click: &mut ClickEvent,
    ) {
        Window::new("Model Hierarchy")
            .enabled(data_source.input_mode.is_interact_ui())
            .open(&mut data_source.is_model_hierarchy_open)
            .show(context, |ui| {
                if let Some(model_view_data) = data_source.model_view_data.as_ref() {
                    Self::render_collapsing_header(
                        ui,
                        &model_view_data.mesh_items,
                        &model_view_data.file_path,
                        click,
                    );
                }
            });
    }

    fn render_collapsing_header(
        ui: &mut Ui,
        mesh_items: &[Rc<MeshItem>],
        file_path: &std::path::Path,
        click: &mut ClickEvent,
    ) {
        for mesh_item in mesh_items {
            let id = ui.make_persistent_id(mesh_item.name.clone());
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    if ui.button(mesh_item.name.clone()).clicked() {
                        click.mesh_item = Some(ClickMeshItem {
                            item: mesh_item.clone(),
                            file_path: file_path.to_path_buf(),
                        });
                    }
                })
                .body(|ui| {
                    Self::render_collapsing_header(ui, &mesh_item.childs, file_path, click);
                });
        }
    }

    fn new_window(name: &str, input_mode: EInputMode) -> egui::Window<'static> {
        Window::new(name)
            .enabled(input_mode.is_interact_ui())
            .interactable(input_mode.is_interact_ui())
    }
}
