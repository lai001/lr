use crate::{
    custom_event::{ECustomEventType, EFileDialogType},
    data_source::DataSource,
    editor_ui::EditorUI,
    model_loader::ModelLoader,
    project::{Project, ProjectContext},
};
use std::{path::PathBuf, process::Command};
use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};

const MODEL_EXTENSION: [&str; 1] = ["fbx"];
const IMAGE_EXTENSION: [&str; 2] = ["png", "jpg"];

pub struct EditorContext {
    engine: rs_engine::engine::Engine,
    platform: egui_winit_platform::Platform,
    data_source: DataSource,
    project_context: Option<ProjectContext>,
}

impl EditorContext {
    fn load_font() -> egui::FontDefinitions {
        let font_path =
            std::path::Path::new("./Font/SimplifiedChineseHW/SourceHanSansHWSC-Regular.otf");
        let font_data = match std::fs::read(font_path) {
            Ok(font_data) => font_data,
            Err(_) => {
                return egui::FontDefinitions::default();
            }
        };
        let mut font_definitions = egui::FontDefinitions::default().clone();
        font_definitions.families.clear();
        font_definitions.font_data.clear();
        font_definitions.font_data.insert(
            "SourceHanSansHWSC-Regular".to_owned(),
            egui::FontData::from_owned(font_data),
        );

        font_definitions.families.insert(
            egui::FontFamily::Monospace,
            vec!["SourceHanSansHWSC-Regular".to_owned()],
        );

        font_definitions.families.insert(
            egui::FontFamily::Proportional,
            vec!["SourceHanSansHWSC-Regular".to_owned()],
        );
        font_definitions
    }

    pub fn new(window: &winit::window::Window) -> Self {
        rs_foundation::change_working_directory();

        let window_size = window.inner_size();
        let scale_factor = 1.0f32;
        let window_width = window_size.width;
        let window_height = window_size.height;
        let descriptor = egui_winit_platform::PlatformDescriptor {
            physical_width: window_width,
            physical_height: window_height,
            scale_factor: scale_factor as f64,
            font_definitions: Self::load_font(),
            style: egui::Style::default(),
        };
        let platform = egui_winit_platform::Platform::new(descriptor);
        let artifact_reader = None;
        let engine = rs_engine::engine::Engine::new(
            window,
            window_width,
            window_height,
            scale_factor,
            platform.context(),
            artifact_reader,
        )
        .unwrap();
        let data_source = DataSource::new();

        Self {
            engine,
            platform,
            data_source,
            project_context: None,
        }
    }

    pub fn handle_event(
        &mut self,
        window: &mut winit::window::Window,
        event: &Event<ECustomEventType>,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
        control_flow: &mut ControlFlow,
    ) {
        self.platform.handle_event(&event);

        match event {
            Event::UserEvent(event) => {
                self.process_custom_event(event, window);
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    log::trace!("Window resized: {:?}", size);
                    self.engine.resize(size.width, size.height);
                }
                WindowEvent::KeyboardInput {
                    input,
                    is_synthetic,
                    ..
                } => {
                    self.process_keyboard_input(&input, *is_synthetic);
                }
                WindowEvent::DroppedFile(file_path) => {
                    self.process_import_asset(file_path.to_owned());
                }
                WindowEvent::Ime(ime) => {
                    match ime {
                        winit::event::Ime::Enabled | winit::event::Ime::Disabled => (),
                        winit::event::Ime::Commit(text) => {
                            self.data_source.input_method_editor_started = false;
                            self.platform
                                .raw_input_mut()
                                .events
                                .push(egui::Event::CompositionEnd(text.clone()));
                        }
                        winit::event::Ime::Preedit(text, Some(_)) => {
                            if !self.data_source.input_method_editor_started {
                                self.data_source.input_method_editor_started = true;
                                self.platform
                                    .raw_input_mut()
                                    .events
                                    .push(egui::Event::CompositionStart);
                            }
                            self.platform
                                .raw_input_mut()
                                .events
                                .push(egui::Event::CompositionUpdate(text.clone()));
                        }
                        winit::event::Ime::Preedit(_, None) => {}
                    };
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                self.process_redraw_request(control_flow, event_loop_proxy);
            }
            Event::RedrawEventsCleared => {
                if let Some(context) = self.project_context.as_mut() {
                    context.reload_if_need();
                }
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn process_keyboard_input(&mut self, input: &KeyboardInput, is_synthetic: bool) {}

    fn process_custom_event(
        &mut self,
        event: &ECustomEventType,
        window: &mut winit::window::Window,
    ) {
        match event {
            ECustomEventType::OpenFileDialog(dialog_type) => match dialog_type {
                EFileDialogType::NewProject(name) => {
                    if name.is_empty() || name.len() > 127 {
                        return;
                    }
                    let dialog = rfd::FileDialog::new();
                    if let Some(folder) = dialog.pick_folder() {
                        log::trace!("Selected folder: {:?}", folder);
                        match Project::create_empty_project(&folder, name) {
                            Ok(project_file_path) => {
                                if let Ok(project_context) = Project::open(&project_file_path) {
                                    self.project_context = Some(project_context);
                                    window.set_title(&format!("Editor({})", name));
                                    std::thread::spawn(move || {
                                        let arg =
                                            project_file_path.parent().unwrap().to_str().unwrap();
                                        let _ = Command::new("Code")
                                            .arg(arg)
                                            .stdout(std::process::Stdio::null())
                                            .stderr(std::process::Stdio::null())
                                            .spawn();
                                    });
                                }
                            }
                            Err(err) => {
                                log::warn!("{:?}", err);
                            }
                        }
                        self.data_source.is_new_project_window_open = false;
                    }
                }
                EFileDialogType::OpenProject => {
                    let dialog = rfd::FileDialog::new().add_filter("Project", &["rsproject"]);
                    if let Some(file_path) = dialog.pick_file() {
                        log::trace!("Selected file: {:?}", file_path);
                        if let Ok(project_context) = Project::open(&file_path) {
                            window.set_title(&format!(
                                "Editor({})",
                                project_context.project.project_name
                            ));
                            self.project_context = Some(project_context);
                            std::thread::spawn(move || {
                                let arg = file_path.parent().unwrap().to_str().unwrap();
                                let _ = Command::new("Code")
                                    .arg(arg)
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .spawn();
                            });
                        }
                    }
                }
                EFileDialogType::ImportAsset => {
                    let mut filter = MODEL_EXTENSION.to_vec();
                    filter.append(&mut IMAGE_EXTENSION.to_vec());
                    let dialog = rfd::FileDialog::new().add_filter("Asset", &filter);
                    if let Some(file_path) = dialog.pick_file() {
                        self.process_import_asset(file_path);
                    }
                }
            },
        }
    }

    fn process_import_asset(&mut self, file_path: PathBuf) {
        log::trace!("Selected file: {:?}", file_path);
        let Some(extension) = file_path.extension() else {
            return;
        };
        let extension = extension.to_str().unwrap();

        if MODEL_EXTENSION.contains(&extension) {
            self.process_import_model(file_path.clone(), extension);
        } else if IMAGE_EXTENSION.contains(&extension) {
            self.process_import_image(file_path.clone(), extension);
        }
    }

    fn process_import_image(&mut self, file_path: PathBuf, extension: &str) {
        let image = image::open(file_path);
        if let Ok(image) = image {
            log::trace!("Width: {}, Height: {}", image.width(), image.height());
        }
    }

    fn process_import_model(&mut self, file_path: PathBuf, extension: &str) {
        match extension {
            "fbx" => {
                let mesh_clusters = ModelLoader::load_from_file(file_path.to_str().unwrap(), &[]);
                for mesh_cluster in mesh_clusters {
                    log::trace!("Vertex: {}", mesh_cluster.vertex_buffer.len());
                    log::trace!("Index: {}", mesh_cluster.index_buffer.len());
                    log::trace!("Texture: {}", mesh_cluster.textures_dic.len());
                    let static_mesh = rs_artifact::static_mesh::StaticMesh {
                        name: "".to_string(),
                        id: uuid::Uuid::new_v4(),
                        vertexes: mesh_cluster.vertex_buffer,
                        indexes: mesh_cluster.index_buffer,
                        url: rs_artifact::default_url().clone(),
                    };
                    let serialize_data = bincode::serialize(&static_mesh).unwrap_or(vec![]);
                    log::trace!("Data length: {}", serialize_data.len());
                }
            }
            _ => {}
        }
    }

    fn process_redraw_request(
        &mut self,
        control_flow: &mut ControlFlow,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    ) {
        let elapsed = std::time::Instant::now() - self.data_source.current_frame_start_time;
        Self::sync_fps(elapsed, self.data_source.target_fps, control_flow);
        self.data_source.current_frame_start_time = std::time::Instant::now();

        self.platform.begin_frame();
        let click_event = EditorUI::build(&self.platform.context(), &mut self.data_source);

        {
            if let Some(context) = self.project_context.as_mut() {
                let lib = context.hot_reload.get_library_reload();
                let lib = lib.lock().unwrap();
                if let Ok(func) = lib.load_symbol::<fn(&egui::Context)>("render") {
                    func(&self.platform.context());
                }
            }
        }

        if click_event.is_new_project {
            let _ = event_loop_proxy.send_event(ECustomEventType::OpenFileDialog(
                EFileDialogType::NewProject(self.data_source.new_project_name.clone()),
            ));
        }
        if click_event.is_open_project {
            let _ = event_loop_proxy.send_event(ECustomEventType::OpenFileDialog(
                EFileDialogType::OpenProject,
            ));
        }
        if click_event.is_import_asset {
            let _ = event_loop_proxy.send_event(ECustomEventType::OpenFileDialog(
                EFileDialogType::ImportAsset,
            ));
        }
        let full_output = self.platform.end_frame(None);
        self.engine.redraw(full_output);
    }

    fn sync_fps(
        elapsed: std::time::Duration,
        fps: u64,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) {
        let fps = std::time::Duration::from_secs_f32(1.0 / fps as f32);
        let wait: std::time::Duration;
        if fps < elapsed {
            wait = std::time::Duration::from_millis(0);
        } else {
            wait = fps - elapsed;
        }
        let new_inst = std::time::Instant::now() + wait;
        *control_flow = winit::event_loop::ControlFlow::WaitUntil(new_inst);
    }
}
