use std::{path::PathBuf, rc::Rc};

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

#[derive(Debug)]
pub struct AssetFile {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct AssetFolder {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<AssetFile>,
    pub folders: Vec<AssetFolder>,
}

#[derive(Debug)]
pub struct Node {
    pub id: uuid::Uuid,
    pub name: String,
    pub childs: Vec<Node>,
}

#[derive(Debug)]
pub struct Level {
    pub name: String,
    pub nodes: Vec<Node>,
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
    pub model_view_data: Option<ModelViewData>,
    pub is_level_view_open: bool,
    pub level: Option<Level>,
    pub is_cursor_visible: bool,
    pub camera_movement_speed: f32,
    pub camera_motion_speed: f32,
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
            is_asset_folder_open: false,
            asset_folder: None,
            is_model_hierarchy_open: false,
            model_view_data: None,
            is_level_view_open: false,
            level: None,
            is_cursor_visible: true,
            camera_movement_speed: 0.01,
            camera_motion_speed: 0.1,
        }
    }

    pub fn get_app_start_time(&self) -> std::time::Instant {
        self.app_start_time
    }
}
