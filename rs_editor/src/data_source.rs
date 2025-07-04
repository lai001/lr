use crate::{
    project_context::RecentProjects,
    standalone_simulation_options::MultiplePlayerOptions,
    ui::{content_browser, curve_view::CurveViewDataSource, model_scene_view},
};
use rs_core_minimal::settings::Settings;
use rs_engine::{
    console_cmd::ConsoleCmd, content::curve::Curve, file_type::EFileType, input_mode::EInputMode,
};
use rs_foundation::new::SingleThreadMutType;
use rs_render::bake_info::BakeInfo;
use rs_render::view_mode::EViewModeType;
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct MeshItem {
    pub name: String,
    pub childs: Vec<Rc<MeshItem>>,
}

#[derive(Debug)]
pub struct ModelViewData {
    pub mesh_items: Vec<Rc<MeshItem>>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AssetFile {
    pub name: String,
    pub path: PathBuf,
}

impl AssetFile {
    pub fn get_file_type(&self) -> EFileType {
        EFileType::from_path(&self.path).expect("Supported file type.")
    }
}

#[derive(Debug, Clone)]
pub struct AssetFolder {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<AssetFile>,
    pub folders: Vec<AssetFolder>,
}

pub struct DataSource {
    pub target_fps: u64,
    pub current_frame_start_time: std::time::Instant,
    app_start_time: std::time::Instant,
    pub is_file_dialog_open: bool,
    pub is_new_project_window_open: bool,
    pub new_project_name: String,
    pub input_method_editor_started: bool,
    pub is_asset_folder_open: bool,
    pub asset_folder: Option<AssetFolder>,
    pub current_asset_folder: Option<AssetFolder>,
    pub highlight_asset_file: Option<AssetFile>,
    pub is_level_view_open: bool,
    pub level: Option<Rc<RefCell<rs_engine::content::level::Level>>>,
    pub camera_view_matrix: glam::Mat4,
    pub camera_projection_matrix: glam::Mat4,
    pub content_data_source: content_browser::DataSource,
    pub project_settings: Option<Rc<RefCell<Settings>>>,
    pub project_settings_open: bool,
    pub ibl_bake_info: BakeInfo,
    pub recent_projects: RecentProjects,
    pub input_mode: EInputMode,
    pub view_mode: EViewModeType,
    pub console_cmds: Option<Rc<RefCell<HashMap<String, SingleThreadMutType<ConsoleCmd>>>>>,
    pub is_console_cmds_view_open: bool,
    pub is_content_item_property_view_open: bool,
    pub is_object_property_view_open: bool,
    pub debug_shading_type: rs_render::global_uniform::EDebugShadingType,
    pub debug_flags: rs_engine::player_viewport::DebugFlags,
    pub is_debug_texture_view_open: bool,
    pub is_simulate_real_time: bool,
    pub model_scene_view_data: model_scene_view::DataSource,
    pub opened_curve: Option<SingleThreadMutType<Curve>>,
    pub curve_data_source: CurveViewDataSource,
    pub is_gizmo_focused: bool,
    pub is_gizmo_setting_open: bool,
    pub is_show_debug: bool,
    pub multiple_players: u32,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            target_fps: 60,
            current_frame_start_time: std::time::Instant::now(),
            app_start_time: std::time::Instant::now(),
            is_file_dialog_open: false,
            is_new_project_window_open: false,
            new_project_name: String::new(),
            input_method_editor_started: false,
            is_asset_folder_open: true,
            asset_folder: None,
            is_level_view_open: true,
            level: None,
            current_asset_folder: None,
            highlight_asset_file: None,
            camera_view_matrix: glam::Mat4::IDENTITY,
            camera_projection_matrix: glam::Mat4::IDENTITY,
            project_settings: None,
            project_settings_open: false,
            ibl_bake_info: Default::default(),
            content_data_source: content_browser::DataSource::new(),
            recent_projects: RecentProjects::load(),
            input_mode: EInputMode::UI,
            view_mode: EViewModeType::Lit,
            console_cmds: None,
            is_console_cmds_view_open: false,
            is_content_item_property_view_open: false,
            is_object_property_view_open: false,
            debug_shading_type: rs_render::global_uniform::EDebugShadingType::None,
            is_debug_texture_view_open: false,
            is_simulate_real_time: false,
            debug_flags: rs_engine::player_viewport::DebugFlags::empty(),
            model_scene_view_data: model_scene_view::DataSource::default(),
            opened_curve: None,
            curve_data_source: CurveViewDataSource::default(),
            is_gizmo_focused: false,
            is_gizmo_setting_open: false,
            is_show_debug: true,
            multiple_players: MultiplePlayerOptions::default().players,
        }
    }

    pub fn get_app_start_time(&self) -> std::time::Instant {
        self.app_start_time
    }
}
