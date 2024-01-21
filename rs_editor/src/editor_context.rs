use crate::{
    camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle},
    custom_event::{ECustomEventType, EFileDialogType},
    data_source::{AssetFile, AssetFolder, DataSource, MeshItem, ModelViewData},
    editor_ui::EditorUI,
    level::MeshReference,
    model_loader::ModelLoader,
    project::Project,
    project_context::{EFolderUpdateType, ProjectContext},
};
use rs_engine::camera::Camera;
use rs_render::command::{DrawObject, PhongMaterial};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

const FBX_EXTENSION: &str = "fbx";
const PNG_EXTENSION: &str = "png";
const JPG_EXTENSION: &str = "jpg";
const MODEL_EXTENSION: [&str; 1] = [FBX_EXTENSION];
const IMAGE_EXTENSION: [&str; 2] = [PNG_EXTENSION, JPG_EXTENSION];
const SUPPORT_ASSET_FILE_EXTENSIONS: [&str; 3] = [FBX_EXTENSION, PNG_EXTENSION, JPG_EXTENSION];

pub struct EditorContext {
    engine: rs_engine::engine::Engine,
    platform: egui_winit_platform::Platform,
    data_source: DataSource,
    project_context: Option<ProjectContext>,
    draw_objects: Vec<DrawObject>,
    camera: Camera,
    virtual_key_code_states: HashMap<winit::event::VirtualKeyCode, winit::event::ElementState>,
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
        let camera = Camera::default(window_width, window_height);
        Self {
            engine,
            platform,
            data_source,
            project_context: None,
            draw_objects: Vec::new(),
            camera,
            virtual_key_code_states: HashMap::new(),
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
            Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    DefaultCameraInputEventHandle::mouse_motion_handle(
                        &mut self.camera,
                        *delta,
                        self.data_source.is_cursor_visible,
                        self.data_source.camera_motion_speed,
                    );
                }
                _ => {}
            },
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
                    self.process_keyboard_input(window, control_flow, &input, *is_synthetic);
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
                if let Some(project_context) = &mut self.project_context {
                    if let Some(folder_update_type) = project_context.check_folder_notification() {
                        match folder_update_type {
                            EFolderUpdateType::Asset => {
                                let asset_folder = Self::build_asset_folder(
                                    &project_context.get_asset_folder_path(),
                                );
                                log::trace!("Update asset folder. {:?}", asset_folder);
                                self.data_source.asset_folder = Some(asset_folder);
                            }
                        }
                    }
                }
                self.control_fps(control_flow);
                for (virtual_key_code, element_state) in &self.virtual_key_code_states {
                    DefaultCameraInputEventHandle::keyboard_input_handle(
                        &mut self.camera,
                        virtual_key_code,
                        element_state,
                        self.data_source.is_cursor_visible,
                        self.data_source.camera_movement_speed,
                    );
                }
                self.camera_did_update();
                self.process_redraw_request(event_loop_proxy);
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

    fn build_asset_folder(path: &std::path::Path) -> AssetFolder {
        let mut folder = AssetFolder {
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
            path: path.to_path_buf(),
            files: vec![],
            folders: vec![],
        };

        for entry in walkdir::WalkDir::new(path).max_depth(1) {
            if let Ok(entry) = entry {
                if entry.path() == path {
                    continue;
                }
                if entry.path().is_dir() {
                    let path = entry.path();
                    let sub_folder = Self::build_asset_folder(path);
                    folder.folders.push(sub_folder);
                } else {
                    if let Some(extension) = entry.path().extension() {
                        if SUPPORT_ASSET_FILE_EXTENSIONS.contains(&extension.to_str().unwrap()) {
                            let asset_file = AssetFile {
                                name: entry.file_name().to_str().unwrap().to_string(),
                                path: entry.path().to_path_buf(),
                            };
                            folder.files.push(asset_file);
                        }
                    }
                }
            }
        }
        folder
    }

    fn process_keyboard_input(
        &mut self,
        window: &mut winit::window::Window,
        control_flow: &mut ControlFlow,
        input: &KeyboardInput,
        is_synthetic: bool,
    ) {
        let Some(virtual_keycode) = input.virtual_keycode else {
            return;
        };

        self.virtual_key_code_states
            .insert(virtual_keycode, input.state);

        if Self::is_keys_pressed(
            &mut self.virtual_key_code_states,
            &[VirtualKeyCode::F1],
            true,
        ) {
            self.data_source.is_cursor_visible = !self.data_source.is_cursor_visible;
            if self.data_source.is_cursor_visible {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap();
            } else {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .unwrap();
            }
            window.set_cursor_visible(self.data_source.is_cursor_visible);
        }

        if Self::is_keys_pressed(
            &mut self.virtual_key_code_states,
            &[VirtualKeyCode::LAlt, VirtualKeyCode::F4],
            true,
        ) {
            *control_flow = ControlFlow::Exit;
        }
    }

    fn is_keys_pressed(
        virtual_key_code_states: &mut HashMap<VirtualKeyCode, ElementState>,
        keys: &[VirtualKeyCode],
        is_consume: bool,
    ) -> bool {
        let mut states: HashMap<VirtualKeyCode, ElementState> = HashMap::new();
        for key in keys {
            if let Some(state) = virtual_key_code_states.get(key) {
                states.insert(*key, *state);
            }
        }
        if states.keys().len() == keys.len() {
            for state in states.values() {
                if *state == ElementState::Released {
                    return false;
                }
            }
            if is_consume {
                for key in states.keys() {
                    virtual_key_code_states.remove(key);
                }
            }
            return true;
        } else {
            return false;
        }
    }

    fn is_project_name_valid(name: &str) -> bool {
        if name.is_empty() || name.len() > 127 {
            return false;
        }
        let reg = regex::Regex::new("^[a-zA-Z]*$").unwrap();
        return reg.is_match(name);
    }

    fn process_custom_event(
        &mut self,
        event: &ECustomEventType,
        window: &mut winit::window::Window,
    ) {
        match event {
            ECustomEventType::OpenFileDialog(dialog_type) => match dialog_type {
                EFileDialogType::NewProject(name) => {
                    if Self::is_project_name_valid(&name) == false {
                        return;
                    }
                    let dialog = rfd::FileDialog::new();
                    let Some(folder) = dialog.pick_folder() else {
                        return;
                    };

                    log::trace!("Selected folder: {:?}", folder);
                    let project_file_path = match Project::create_empty_project(&folder, name) {
                        Ok(project_file_path) => project_file_path,
                        Err(err) => {
                            log::warn!("{:?}", err);
                            return;
                        }
                    };
                    let Ok(project_context) = ProjectContext::open(&project_file_path) else {
                        return;
                    };
                    self.project_context = Some(project_context);
                    window.set_title(&format!("Editor({})", name));

                    self.data_source.is_new_project_window_open = false;
                }
                EFileDialogType::OpenProject => {
                    let dialog = rfd::FileDialog::new().add_filter("Project", &["rsproject"]);
                    let Some(file_path) = dialog.pick_file() else {
                        return;
                    };
                    log::trace!("Selected file: {:?}", file_path);

                    let project_context = match ProjectContext::open(&file_path) {
                        Ok(project_context) => project_context,
                        Err(err) => {
                            log::warn!("{:?}", err);
                            return;
                        }
                    };

                    let asset_folder_path = project_context.get_asset_folder_path();
                    let asset_folder = Self::build_asset_folder(&asset_folder_path);
                    log::trace!("Update asset folder. {:?}", asset_folder);
                    self.data_source.asset_folder = Some(asset_folder);
                    window.set_title(&format!("Editor({})", project_context.project.project_name));
                    self.data_source.level = project_context.build_ui_level();
                    self.draw_objects = Self::collect_draw_objects(
                        &mut self.engine,
                        &project_context.get_asset_folder_path(),
                        &self.camera,
                        &project_context.project.level.nodes,
                    );
                    self.project_context = Some(project_context);
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

    pub fn build_asset_url(name: &str) -> Result<url::Url, url::ParseError> {
        url::Url::parse(&format!("asset://{}", name))
    }

    fn open_project_workspace(file_path: std::path::PathBuf) {
        std::thread::spawn(move || {
            let arg = file_path.to_str().unwrap();
            let _ = Command::new("Code")
                .arg(arg)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        });
    }

    fn process_import_model(&mut self, file_path: PathBuf, extension: &str) {
        match extension {
            FBX_EXTENSION => {
                debug_assert!(file_path.is_absolute());
                log::trace!("Open model file: {:?}", file_path);
                let Some(mesh_clusters) = ModelLoader::load_from_file(&file_path, &[]) else {
                    return;
                };
                self.data_source.is_model_hierarchy_open = true;
                let mut items: Vec<Rc<MeshItem>> = vec![];
                for mesh_cluster in mesh_clusters {
                    let item = MeshItem {
                        name: mesh_cluster.name,
                        childs: vec![],
                    };
                    items.push(Rc::new(item));
                }
                let model_view_data = ModelViewData {
                    mesh_items: items,
                    file_path,
                };
                self.data_source.model_view_data = Some(model_view_data);
            }
            _ => {}
        }
    }

    fn control_fps(&mut self, control_flow: &mut ControlFlow) {
        let elapsed = std::time::Instant::now() - self.data_source.current_frame_start_time;
        Self::sync_fps(elapsed, self.data_source.target_fps, control_flow);
        self.data_source.current_frame_start_time = std::time::Instant::now();
    }

    fn process_redraw_request(
        &mut self,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    ) {
        for draw_object in self.draw_objects.clone() {
            self.engine.draw(draw_object);
        }

        let full_output = self.process_ui(event_loop_proxy);
        self.engine.redraw(full_output);
    }

    fn process_ui(
        &mut self,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    ) -> egui::FullOutput {
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
        if click_event.asset_folder {
            self.data_source.is_asset_folder_open = true;
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
        if click_event.is_save_project {
            if let Some(project_context) = self.project_context.as_ref() {
                let save_status = project_context.save();
                log::trace!("Save: {}", save_status);
            }
        }
        if let Some(open_asset_file_path) = click_event.open_asset_file_path {
            let extension = open_asset_file_path.extension().unwrap().to_str().unwrap();
            self.process_import_model(open_asset_file_path.clone(), extension);
        }
        if click_event.is_export {
            if let Some(project_context) = self.project_context.as_mut() {
                let result = project_context.export();
                log::trace!("{:?}", result);
            }
        }
        if let Some(mesh_item) = click_event.mesh_item {
            if let Some(project_context) = self.project_context.as_mut() {
                let file_path = mesh_item
                    .file_path
                    .strip_prefix(project_context.get_asset_folder_path())
                    .unwrap();
                let node = crate::level::Node {
                    name: mesh_item.item.name.clone(),
                    mesh_reference: Some(MeshReference {
                        file_path: file_path.to_path_buf(),
                        referenced_mesh_name: mesh_item.item.name.clone(),
                    }),
                    childs: vec![],
                };
                if let Some(draw_object) = Self::node_to_draw_object(
                    &mut self.engine,
                    &project_context.get_asset_folder_path(),
                    &self.camera,
                    &node,
                ) {
                    self.draw_objects.push(draw_object);
                }
                project_context.project.level.nodes.push(node);
                self.data_source.level = project_context.build_ui_level();
            }
        }
        if click_event.level_window {
            self.data_source.is_level_view_open = true;
        }
        if click_event.open_visual_studio_code {
            if let Some(project_context) = &self.project_context {
                let path = project_context.get_project_folder_path();
                Self::open_project_workspace(path);
            }
        }
        let full_output = self.platform.end_frame(None);
        full_output
    }

    fn node_to_draw_object(
        engine: &mut rs_engine::engine::Engine,
        asset_folder_path: &Path,
        camera: &Camera,
        node: &crate::level::Node,
    ) -> Option<DrawObject> {
        if let Some(mesh_reference) = &node.mesh_reference {
            let constants = rs_render::render_pipeline::phong_pipeline::Constants {
                model: glam::Mat4::IDENTITY,
                view: camera.get_view_matrix(),
                projection: camera.get_projection_matrix(),
            };
            let material = PhongMaterial {
                constants,
                diffuse_texture: None,
                specular_texture: None,
            };

            let mesh_clusters = ModelLoader::load_from_file(
                &asset_folder_path.join(mesh_reference.file_path.clone()),
                &[],
            );
            if let Some(mesh_clusters) = mesh_clusters {
                let mesh_cluster = mesh_clusters
                    .iter()
                    .filter(|x| x.name == mesh_reference.referenced_mesh_name)
                    .next()
                    .unwrap();

                let draw_object = engine.create_draw_object_from_static_mesh(
                    &mesh_cluster.vertex_buffer,
                    &mesh_cluster.index_buffer,
                    rs_render::command::EMaterialType::Phong(material),
                );
                return Some(draw_object);
            }
        }
        return None;
    }

    fn collect_draw_objects(
        engine: &mut rs_engine::engine::Engine,
        asset_folder_path: &Path,
        camera: &Camera,
        nodes: &[crate::level::Node],
    ) -> Vec<DrawObject> {
        let mut draw_objects: Vec<DrawObject> = Vec::new();
        for node in nodes {
            if let Some(draw_object) =
                Self::node_to_draw_object(engine, asset_folder_path, camera, node)
            {
                draw_objects.push(draw_object);
            }
        }
        draw_objects
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

    fn camera_did_update(&mut self) {
        for draw_objects in &mut self.draw_objects {
            match &mut draw_objects.material_type {
                rs_render::command::EMaterialType::Phong(material) => {
                    material.constants.projection = self.camera.get_projection_matrix();
                    material.constants.view = self.camera.get_view_matrix();
                }
                rs_render::command::EMaterialType::PBR(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::EditorContext;

    #[test]
    fn test_case() {
        assert_eq!(EditorContext::is_project_name_valid("name1"), false);
        assert_eq!(EditorContext::is_project_name_valid("1name"), false);
        assert_eq!(EditorContext::is_project_name_valid("*name"), false);
        assert_eq!(EditorContext::is_project_name_valid("name*"), false);
        assert_eq!(EditorContext::is_project_name_valid("*****"), false);
        assert_eq!(EditorContext::is_project_name_valid("11111"), false);
        assert_eq!(EditorContext::is_project_name_valid("na me"), false);

        assert_eq!(EditorContext::is_project_name_valid(""), false);
        assert_eq!(
            EditorContext::is_project_name_valid(&vec!['a'; 999].iter().collect::<String>()),
            false
        );

        assert_eq!(EditorContext::is_project_name_valid("name"), true);
    }
}
