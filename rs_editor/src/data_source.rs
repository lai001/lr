use crate::{project_context::RecentProjects, ui::content_browser};
use rs_core_minimal::settings::Settings;
use rs_engine::{console_cmd::ConsoleCmd, file_type::EFileType, input_mode::EInputMode};
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
    pub is_model_hierarchy_open: bool,
    pub is_asset_folder_open: bool,
    pub asset_folder: Option<AssetFolder>,
    pub current_asset_folder: Option<AssetFolder>,
    pub highlight_asset_file: Option<AssetFile>,
    pub model_view_data: Option<ModelViewData>,
    pub is_level_view_open: bool,
    pub level: Option<Rc<RefCell<rs_engine::content::level::Level>>>,
    pub camera_movement_speed: f32,
    pub camera_motion_speed: f32,
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
    pub current_open_material: Option<SingleThreadMutType<crate::material::Material>>,
    pub is_content_item_property_view_open: bool,
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
            is_model_hierarchy_open: false,
            model_view_data: None,
            is_level_view_open: true,
            level: None,
            camera_movement_speed: 0.01,
            camera_motion_speed: 0.1,
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
            current_open_material: None,
            is_content_item_property_view_open: false,
        }
    }

    pub fn get_app_start_time(&self) -> std::time::Instant {
        self.app_start_time
    }
}
