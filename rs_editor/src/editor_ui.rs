use crate::component_factory::ComponentFactory;
use crate::content_edit::{ContentEdit, UIContentPropertyEvent};
use crate::data_source::{DataSource, MeshItem};
use crate::editor_ui::load::ImageLoader;
use crate::thumbnail_cache::ThumbnailCache;
use crate::ui::component_edit::ComponentEdit;
use crate::ui::content_item_property_view::ContentItemPropertyView;
use crate::ui::debug_textures_view::{self, DebugTexturesView};
use crate::ui::gizmo_view::GizmoView;
use crate::ui::object_property_view::{self, ESelectedObjectType, ObjectPropertyView};
use crate::ui::top_menu::TopMenu;
use crate::ui::{
    asset_view, console_cmds_view, content_browser, curve_view, gizmo_settings, level_view,
    project_settings, top_menu,
};
use egui::*;
use rs_content::Content;
use rs_content_manager::content_manager::ContentManager;
use rs_engine::engine::Engine;
use rs_engine::input_mode::EInputMode;
use rs_foundation::new::SingleThreadMutType;
use rs_localization::t;
use rs_model_loader::model_loader::ModelLoader;
use std::sync::Arc;
use std::{path::PathBuf, rc::Rc};
use transform_gizmo_egui::GizmoResult;
use transform_gizmo_egui::math::Transform;

#[derive(Debug)]
pub struct ClickMeshItem {
    pub file_path: PathBuf,
    pub item: Rc<MeshItem>,
}

pub struct GizmoEvent {
    pub selected_object: ESelectedObjectType,
    pub gizmo_result: Option<(GizmoResult, Vec<Transform>)>,
}

pub struct ContentPropertyViewEvent {
    pub content: SingleThreadMutType<Box<dyn Content>>,
    pub event: Box<dyn UIContentPropertyEvent>,
}

#[derive(Default)]
pub struct ClickEvent {
    pub click_actor: Option<level_view::EClickEventType>,
    pub mesh_item: Option<ClickMeshItem>,
    pub click_aseet: Option<asset_view::EClickItemType>,
    pub menu_event: Option<top_menu::EClickEventType>,
    pub content_browser_event: Option<content_browser::EClickEventType>,
    pub debug_textures_view_event: Option<debug_textures_view::EClickEventType>,
    pub project_settings_event: Option<project_settings::EEventType>,
    pub object_property_view_event: Option<object_property_view::EEventType>,
    pub content_property_view_event: Option<ContentPropertyViewEvent>,
    pub gizmo_event: Option<GizmoEvent>,
}

pub struct EditorUI {
    _image_loader: Option<Arc<dyn ImageLoader + Send + Sync + 'static>>,
    _svg_loader: Option<Arc<dyn ImageLoader + Send + Sync + 'static>>,
    project_folder_path: Option<PathBuf>,
    top_menu: TopMenu,
    pub gizmo_view: GizmoView,
    pub egui_context: Context,
    pub content_item_property_view: ContentItemPropertyView,
    pub object_property_view: ObjectPropertyView,
    pub debug_textures_view: DebugTexturesView,
    thumbnail_cache: ThumbnailCache,
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
        EditorUI {
            _image_loader: image_loader,
            _svg_loader: svg_loader,
            project_folder_path: None,
            top_menu: TopMenu {
                new_project_name: String::new(),
            },
            gizmo_view: GizmoView::default(),
            egui_context: context.clone(),
            content_item_property_view: ContentItemPropertyView::new(),
            object_property_view: ObjectPropertyView::new(),
            debug_textures_view: DebugTexturesView::new(),
            thumbnail_cache: ThumbnailCache::new(),
        }
    }

    pub fn set_project_folder_path(&mut self, project_folder_path: Option<PathBuf>) {
        self.project_folder_path = project_folder_path;
    }

    pub fn build(
        &mut self,
        context: &Context,
        data_source: &mut DataSource,
        model_loader: &mut ModelLoader,
        component_factory: &ComponentFactory,
        component_edit: &mut ComponentEdit,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
        content_edit: &mut ContentEdit,
    ) -> ClickEvent {
        let mut click = ClickEvent::default();

        if let Some(level) = &data_source.level {
            let window = Self::new_window(t!("Level"), "Level", data_source.input_mode);
            click.click_actor = crate::ui::level_view::draw(
                window,
                context,
                &mut data_source.is_level_view_open,
                &level.borrow(),
                component_factory,
            );
        }
        let window = Self::new_window(t!("Asset"), "Asset", data_source.input_mode);
        click.click_aseet = asset_view::draw(
            window,
            context,
            &mut data_source.is_asset_folder_open,
            data_source.current_asset_folder.as_ref(),
            data_source.highlight_asset_file.as_ref(),
            &mut self.thumbnail_cache,
            content_edit,
        );

        if let Some(selected_object) = self.object_property_view.selected_object.as_ref() {
            let model_matrix = match selected_object {
                ESelectedObjectType::Actor(_) => None,
                ESelectedObjectType::SceneNode(scene_node) => {
                    let scene_node = scene_node.borrow();
                    let final_transformation =
                        Some(scene_node.component().get_final_transformation());
                    final_transformation
                }
                ESelectedObjectType::DirectionalLight(component) => {
                    let component = component.borrow();
                    Some(*component.get_transformation())
                }
            };
            if let Some(model_matrix) = model_matrix {
                let gizmo_result = self.gizmo_view.draw(
                    context,
                    data_source.camera_view_matrix,
                    data_source.camera_projection_matrix,
                    model_matrix,
                );
                click.gizmo_event = Some(GizmoEvent {
                    selected_object: selected_object.clone(),
                    gizmo_result,
                });
                data_source.is_gizmo_focused = self.gizmo_view.is_focused();
            } else {
                data_source.is_gizmo_focused = false;
            }
        }

        // Fix gizmo rendering causing top menu flickering, draw menu above the gizmo view
        let mut panel_ui = rs_egui_utils::create_panel_ui_from_context(
            context,
            Some(egui::Id::new("EditorPanel")),
        );
        click.menu_event = self.top_menu.draw(context, &mut panel_ui, data_source);

        let window = Self::new_window(
            t!("Gizmo Settings"),
            "Gizmo Settings",
            data_source.input_mode,
        );
        gizmo_settings::draw(
            window,
            context,
            &mut self.gizmo_view.visuals,
            &mut self.gizmo_view.gizmo_mode,
            &mut self.gizmo_view.gizmo_orientation,
            &mut self.gizmo_view.custom_highlight_color,
            &mut data_source.is_gizmo_setting_open,
        );
        if let Some(project_settings) = data_source.project_settings.clone() {
            let window = Self::new_window(
                t!("Project Settings"),
                "Project Settings",
                data_source.input_mode,
            );
            click.project_settings_event = crate::ui::project_settings::draw(
                window,
                context,
                &mut data_source.project_settings_open,
                project_settings,
                data_source.content_data_source.contents.clone(),
            );
        }
        if let Some(project_folder_path) = self.project_folder_path.as_ref() {
            let window = Self::new_window(
                t!("Content Browser"),
                "Content Browser",
                data_source.input_mode,
            );
            click.content_browser_event = content_browser::draw(
                window,
                context,
                project_folder_path,
                &mut data_source.content_data_source,
                &mut self.thumbnail_cache,
                content_edit, // data_source.input_mode,
            );
        }
        if let Some(console_cmds) = &data_source.console_cmds {
            let window =
                Self::new_window(t!("Console Cmds"), "Console Cmds", data_source.input_mode);
            console_cmds_view::draw(
                window,
                context,
                &mut data_source.is_console_cmds_view_open,
                &mut console_cmds.borrow_mut(),
            );
        }

        Self::new_window(
            t!("Content Property"),
            "Content Property",
            data_source.input_mode,
        )
        .open(&mut data_source.is_content_item_property_view_open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([250.0, 500.0])
        .show(context, |ui| {
            let event = self.content_item_property_view.draw(ui, content_edit);
            if let Some(content) = self.content_item_property_view.content.clone() {
                if let Some(event) = event {
                    click.content_property_view_event =
                        Some(ContentPropertyViewEvent { content, event });
                }
            }
        });

        Self::new_window(t!("Detail"), "Detail", data_source.input_mode)
            .open(&mut data_source.is_object_property_view_open)
            .vscroll(true)
            .hscroll(true)
            .resizable(true)
            .default_size([250.0, 500.0])
            .show(context, |ui| {
                click.object_property_view_event =
                    self.object_property_view
                        .draw(ui, component_edit, engine, content_manager);
            });

        Self::new_window(
            t!("Debug Texture View"),
            "Debug Texture View",
            data_source.input_mode,
        )
        .open(&mut data_source.is_debug_texture_view_open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([500.0, 500.0])
        .show(context, |ui| {
            click.debug_textures_view_event = self.debug_textures_view.draw(ui);
        });

        let mut is_open = data_source.model_scene_view_data.model_scene.is_some();
        let mut scene = None;
        if let Some(path) = data_source.model_scene_view_data.model_scene.clone() {
            scene = model_loader.get(&path);
        }
        Self::new_window(t!("Model Scene"), "Model Scene", data_source.input_mode)
            .open(&mut is_open)
            .vscroll(true)
            .hscroll(true)
            .resizable(true)
            .default_size([500.0, 500.0])
            .show(context, |ui| {
                if let Some(scene) = scene {
                    crate::ui::model_scene_view::render(
                        ui,
                        scene.as_ref(),
                        &mut data_source.model_scene_view_data,
                    );
                }
            });
        if !is_open {
            data_source.model_scene_view_data.model_scene = None;
        }

        let mut is_curve_open = true;
        if let Some(opend_curve) = data_source.opened_curve.clone() {
            let mut opend_curve = opend_curve.borrow_mut();
            let name = opend_curve.get_name();
            Self::new_window(
                t!("Curve %{name}", name = name),
                format!("Curve({})", name),
                data_source.input_mode,
            )
            .open(&mut is_curve_open)
            .vscroll(false)
            .hscroll(false)
            .resizable(true)
            .default_size([500.0, 500.0])
            .show(context, |ui| {
                curve_view::draw(&mut opend_curve, ui, &mut data_source.curve_data_source);
            });
        }
        if !is_curve_open {
            data_source.opened_curve = None;
        }

        click
    }

    pub fn new_window(
        name: impl Into<WidgetText>,
        id_source: impl std::hash::Hash,
        input_mode: EInputMode,
    ) -> egui::Window<'static> {
        Window::new(name)
            .id(egui::Id::new(id_source))
            .enabled(input_mode.is_interact_ui())
            .interactable(input_mode.is_interact_ui())
    }
}
