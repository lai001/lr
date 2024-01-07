pub struct DataSource {
    pub target_fps: u64,
    pub current_frame_start_time: std::time::Instant,
    app_start_time: std::time::Instant,
    pub is_file_dialog_open: bool,
    pub is_new_project_window_open: bool,
    pub new_project_name: String,
    pub input_method_editor_started: bool,
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
        }
    }

    pub fn get_app_start_time(&self) -> std::time::Instant {
        self.app_start_time
    }
}
