use crate::{
    build_config::EBuildType,
    content_folder::ContentFolder,
    custom_event::{ECustomEventType, EFileDialogType},
    data_source::{AssetFile, AssetFolder, DataSource},
    editor_ui::{EditorUI, GizmoEvent},
    material_resolve,
    project::Project,
    project_context::{EFolderUpdateType, ProjectContext},
    standalone_simulation_options::{
        MultiplePlayerOptions, StandaloneSimulationType, DEFAULT_SERVER_ADDR,
    },
    ui::{
        asset_view,
        blend_animations_ui_window::BlendAnimationUIWindow,
        content_browser, content_item_property_view, debug_textures_view,
        material_ui_window::MaterialUIWindow,
        material_view::{self, EMaterialNodeType, MaterialNode},
        media_ui_window::MediaUIWindow,
        mesh_ui_window::MeshUIWindow,
        misc::update_window_with_input_mode,
        multiple_draw_ui_window::MultipleDrawUiWindow,
        object_property_view::{self, ESelectedObjectType},
        particle_system_ui_window::ParticleSystemUIWindow,
        standalone_ui_window::StandaloneUiWindow,
        top_menu,
        ui_window::UIWindow,
    },
    watch_shader::WatchShader,
    windows_manager::WindowsManager,
};
use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use rs_core_minimal::{
    file_manager, name_generator::make_unique_name, path_ext::CanonicalizeSlashExt,
};
#[cfg(any(feature = "plugin_shared_crate"))]
use rs_engine::plugin::plugin_crate::Plugin;
use rs_engine::{
    build_asset_url, build_built_in_resouce_url, build_content_file_url,
    camera_component::CameraComponent,
    collision_componenet::CollisionComponent,
    components::{
        component::Component, point_light_component::PointLightComponent,
        spot_light_component::SpotLightComponent,
    },
    content::{
        blend_animations::BlendAnimations, content_file_type::EContentFileType,
        texture::TextureFile,
    },
    directional_light::DirectionalLight,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
    logger::SlotFlags,
    player_viewport::PlayerViewport,
    scene_node::SceneNode,
    url_extension::UrlExtension,
};
use rs_engine::{
    file_type::EFileType,
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_metis::{cluster::ClusterCollection, vertex_position::VertexPosition};
use rs_model_loader::model_loader::ModelLoader;
use rs_render::{
    command::{RenderCommand, ScaleChangedInfo, TextureDescriptorCreateInfo},
    get_buildin_shader_dir,
};
use rs_render_types::MaterialOptions;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::Arc,
};
use transform_gizmo_egui::{GizmoMode, GizmoOrientation};
use winit::{
    event::{ElementState, Event, WindowEvent},
    keyboard::KeyCode,
};

lazy_static! {
    static ref SUPPORT_ASSET_IMAGE_FILE_TYPES: HashSet<EFileType> = {
        let mut m = HashSet::new();
        m.insert(EFileType::Jpeg);
        m.insert(EFileType::Jpg);
        m.insert(EFileType::Png);
        m.insert(EFileType::Exr);
        m.insert(EFileType::Hdr);
        m
    };
    static ref SUPPORT_ASSET_MODEL_FILE_TYPES: HashSet<EFileType> = {
        let mut m = HashSet::new();
        m.insert(EFileType::Fbx);
        m.insert(EFileType::Glb);
        m.insert(EFileType::Blend);
        m.insert(EFileType::Dae);
        m
    };
    static ref SUPPORT_ASSET_FILE_TYPES: HashSet<EFileType> = {
        let mut m = HashSet::new();
        m.extend(SUPPORT_ASSET_IMAGE_FILE_TYPES.iter());
        m.extend(SUPPORT_ASSET_MODEL_FILE_TYPES.iter());
        m.extend(SUPPORT_ASSET_MEDIA_FILE_TYPES.iter());
        m.extend(SUPPORT_ASSET_SOUND_FILE_TYPES.iter());
        m
    };
    static ref SUPPORT_ASSET_MEDIA_FILE_TYPES: HashSet<EFileType> = {
        let mut m = HashSet::new();
        m.insert(EFileType::Mp4);
        m
    };
    static ref SUPPORT_ASSET_SOUND_FILE_TYPES: HashSet<EFileType> = {
        let mut m = HashSet::new();
        m.insert(EFileType::WAV);
        m.insert(EFileType::MP3);
        m
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EWindowType {
    Main,
    Material,
    Mesh,
    Media,
    MultipleDraw,
    Particle,
    Standalone,
    Actor,
    BlendAnimation,
}

struct MouseState {
    is_focus: bool,
    position: glam::Vec2,
}

#[cfg(feature = "plugin_v8")]
struct V8Plugin {
    runtime: rs_v8_host::v8_runtime::V8Runtime,
    binding_api_manager: rs_v8_binding_api_manager::BindingApiManager,
}

pub struct EditorContext {
    event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    engine: rs_engine::engine::Engine,
    egui_winit_state: egui_winit::State,
    data_source: DataSource,
    project_context: Option<ProjectContext>,
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    editor_ui: EditorUI,
    // #[cfg(any(feature = "plugin_shared_crate"))]
    // plugins: Vec<Box<dyn Plugin>>,
    #[cfg(feature = "plugin_v8")]
    v8_plugin: Option<V8Plugin>,
    frame_sync: FrameSync,
    model_loader: ModelLoader,
    window_manager: Rc<RefCell<WindowsManager>>,
    material_ui_window: Option<MaterialUIWindow>,
    particle_system_ui_window: Option<ParticleSystemUIWindow>,
    mesh_ui_window: Option<MeshUIWindow>,
    media_ui_window: Option<MediaUIWindow>,
    multiple_draw_ui_window: Option<MultipleDrawUiWindow>,
    standalone_ui_windows: Vec<StandaloneUiWindow>,
    blend_animation_ui_window: Option<BlendAnimationUIWindow>,
    watch_shader: WatchShader,
    #[cfg(feature = "plugin_dotnet")]
    donet_host: Option<rs_dotnet_host::dotnet_runtime::DotnetRuntime>,
    mosue_state: MouseState,
    player_viewport: PlayerViewport,
}

impl EditorContext {
    fn load_font() -> egui::FontDefinitions {
        let font_path = rs_core_minimal::file_manager::get_engine_resource(
            "Remote/Font/SourceHanSansHWSC/OTF/SimplifiedChineseHW/SourceHanSansHWSC-Regular.otf",
        );
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
            Arc::new(egui::FontData::from_owned(font_data)),
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

    pub fn new(
        window_id: isize,
        window: &winit::window::Window,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
        window_manager: Rc<RefCell<WindowsManager>>,
    ) -> anyhow::Result<EditorContext> {
        let _span = tracy_client::span!();
        rs_foundation::change_working_directory();
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: false,
            is_flush_before_drop: false,
            slot_flags: SlotFlags::Level,
        });
        log::trace!(
            "Engine Root Dir: {:?}",
            rs_core_minimal::file_manager::get_engine_root_dir().canonicalize_slash()?
        );
        log::trace!("Git hash: {}", rs_core_minimal::misc::get_git_hash());
        // for var in std::env::vars() {
        //     log::trace!("{:?}", var);
        // }

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let window_width = window_size.width;
        let window_height = window_size.height;
        let egui_context = egui::Context::default();
        egui_context.set_embed_viewports(false);
        egui_context.set_fonts(Self::load_font());
        let style = egui::Style::default().clone();
        egui_context.set_style(style);
        let egui_winit_state = egui_winit::State::new(
            egui_context,
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let artifact_reader = None;
        let mut engine = rs_engine::engine::Engine::new(
            u64::from(window.id()) as isize,
            window,
            window_width,
            window_height,
            scale_factor,
            logger,
            artifact_reader,
            HashMap::new(),
            ProjectContext::load_shader_naga_modules(),
        )?;

        Self::insert_cmds(&mut engine);

        let mut data_source = DataSource::new();
        data_source.console_cmds = Some(engine.get_console_cmds());
        let editor_ui = EditorUI::new(egui_winit_state.egui_ctx());

        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let watch_shader = WatchShader::new(get_buildin_shader_dir())?;

        #[cfg(feature = "plugin_dotnet")]
        let donet_host = rs_dotnet_host::dotnet_runtime::DotnetRuntime::default().ok();

        let mut player_viewport = PlayerViewport::from_window_surface(
            window_id,
            window_width,
            window_height,
            &mut engine,
            EInputMode::UI,
            true,
        );
        player_viewport.set_name("EditorPlayerViewport".to_string());

        let last_project_path = if engine
            .get_settings()
            .editor_settings
            .is_auto_open_last_project
        {
            data_source.recent_projects.paths.first().cloned()
        } else {
            None
        };

        let mut editor_context = EditorContext {
            event_loop_proxy,
            engine,
            egui_winit_state,
            data_source,
            project_context: None,
            virtual_key_code_states: HashMap::new(),
            editor_ui,
            // #[cfg(feature = "plugin_shared_crate")]
            // plugins: vec![],
            frame_sync,
            model_loader: ModelLoader::new(),
            window_manager: window_manager.clone(),
            material_ui_window: None,
            particle_system_ui_window: None,
            mesh_ui_window: None,
            media_ui_window: None,
            multiple_draw_ui_window: None,
            standalone_ui_windows: Vec::new(),
            blend_animation_ui_window: None,
            watch_shader,
            mosue_state: MouseState {
                is_focus: false,
                position: glam::vec2(0.0, 0.0),
            },
            player_viewport,
            #[cfg(feature = "plugin_dotnet")]
            donet_host,
            #[cfg(feature = "plugin_v8")]
            v8_plugin: None,
        };

        if let Some(file_path) = last_project_path {
            if let Err(err) = editor_context.open_project(&file_path, window) {
                log::warn!("{} {}", err, err.root_cause());
            }
        }

        Ok(editor_context)
    }

    pub fn init_v8(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "plugin_v8")]
        {
            let mut v8_runtime = rs_v8_host::v8_runtime::V8Runtime::new();
            let manager = rs_v8_binding_api_manager::BindingApiManager::new(
                rs_v8_engine_binding_api::native_engine::EngineBindingApi::new(
                    &mut v8_runtime,
                    &mut self.engine,
                )?,
                rs_v8_engine_binding_api::native_level::RcRefLevelBindingApi::new(&mut v8_runtime)?,
                rs_v8_engine_binding_api::native_player_viewport::PlayerViewportBindingApi::new(
                    &mut v8_runtime,
                    &mut self.player_viewport,
                )?,
            );
            v8_runtime.register_func_global()?;
            self.v8_plugin = Some(V8Plugin {
                runtime: v8_runtime,
                binding_api_manager: manager,
            });
            self.v8_plugin
                .as_mut()
                .expect("Not null")
                .runtime
                .associate_embedder_specific_data();
        }
        Ok(())
    }

    fn insert_cmds(engine: &mut rs_engine::engine::Engine) {
        engine.insert_console_cmd(
            rs_engine::console_cmd::RS_TEST_KEY,
            rs_engine::console_cmd::ConsoleCmd {
                key: String::from(rs_engine::console_cmd::RS_TEST_KEY),
                value: rs_engine::console_cmd::EValue::I32(0),
            },
        );
    }

    pub fn main_window_event_process(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        egui_event_response: egui_winit::EventResponse,
    ) {
        match event {
            WindowEvent::CursorEntered { .. } => {
                self.mosue_state.is_focus = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.mosue_state.is_focus = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mosue_state.position.x = position.x as f32;
                self.mosue_state.position.y = position.y as f32;
            }
            WindowEvent::CloseRequested => {
                // #[cfg(feature = "plugin_shared_crate")]
                // self.plugins.clear();
                self.standalone_ui_windows.clear();
                self.egui_winit_state.egui_ctx().memory_mut(|writer| {
                    writer.data.clear();
                });
                if let Some(ctx) = &mut self.project_context {
                    ctx.hot_reload.get_library_reload().lock().unwrap().clear();
                }
                event_loop_window_target.exit();
            }
            WindowEvent::Resized(size) => {
                log::trace!("Main window resized: {:?}", size);
                self.player_viewport
                    .size_changed(size.width, size.height, &mut self.engine);

                self.engine.resize(window_id, size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.engine
                    .send_render_command(RenderCommand::ScaleChanged(ScaleChangedInfo {
                        window_id,
                        new_factor: *scale_factor as f32,
                    }));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.player_viewport
                    .on_window_input(rs_engine::input_type::EInputType::MouseWheel(delta));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == winit::event::MouseButton::Right
                    && !self.egui_winit_state.egui_ctx().is_pointer_over_area()
                {
                    match state {
                        winit::event::ElementState::Pressed => {
                            self.data_source.input_mode = rs_engine::input_mode::EInputMode::Game;
                        }
                        winit::event::ElementState::Released => {
                            self.data_source.input_mode = rs_engine::input_mode::EInputMode::UI;
                        }
                    }
                }
                if *button == winit::event::MouseButton::Left
                    && !self.egui_winit_state.egui_ctx().is_pointer_over_area()
                    && self.mosue_state.is_focus
                    && *state == winit::event::ElementState::Pressed
                    && egui_event_response.consumed == false
                    && self.data_source.is_gizmo_focused == false
                {
                    if let Some(level) = self.data_source.level.as_ref() {
                        let level = level.borrow();
                        let componenet_type = level.ray_cast_find_node(
                            &self.mosue_state.position,
                            &glam::vec2(
                                window.inner_size().width as f32,
                                window.inner_size().height as f32,
                            ),
                            self.player_viewport.camera.get_view_matrix(),
                            self.player_viewport.camera.get_projection_matrix(),
                        );
                        if let Some(componenet_type) = componenet_type {
                            self.editor_ui.object_property_view.selected_object =
                                Some(ESelectedObjectType::SceneNode(componenet_type));
                            self.data_source.is_object_property_view_open = true;
                        }
                        // } else {
                        //     self.editor_ui.object_property_view.selected_object = None;
                        // }
                    }
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                self.process_keyboard_input(
                    device_id,
                    event,
                    *is_synthetic,
                    event_loop_window_target,
                );
            }
            WindowEvent::DroppedFile(file_path) => {
                let Some(project_context) = &self.project_context else {
                    return;
                };
                let target = project_context.get_asset_folder_path();
                let result = std::fs::copy(file_path, target.join(file_path.file_name().unwrap()));
                log::trace!("{:?}", result);
            }
            WindowEvent::RedrawRequested => {
                self.frame_sync.sync(60.0);

                let _span = tracy_client::span!();
                let (is_minimized, is_visible) = {
                    let is_minimized = window.is_minimized().unwrap_or(false);
                    let is_visible = window.is_visible().unwrap_or(true);
                    (is_minimized, is_visible)
                };

                self.engine.tick();
                if !is_visible || is_minimized {
                    return;
                }
                self.engine.window_redraw_requested_begin(window_id);

                let changed_results = self.watch_shader.get_changed_results();
                for changed_result in changed_results {
                    match changed_result {
                        crate::watch_shader::ShaderSourceChangedType::Builtin(changed_result) => {
                            self.engine
                                .send_render_command(RenderCommand::BuiltinShaderChanged(
                                    changed_result,
                                ))
                        }
                        crate::watch_shader::ShaderSourceChangedType::Material => {
                            self.hotreload_material();
                        }
                    }
                }
                #[cfg(feature = "plugin_shared_crate")]
                if let Some(project_context) = &mut self.project_context {
                    if project_context.is_need_reload_plugin() {
                        let mut plugins = Vec::with_capacity(self.standalone_ui_windows.len());
                        for _ in 0..self.standalone_ui_windows.len() {
                            match self.try_create_plugin() {
                                Ok(plugin) => {
                                    plugins.push(plugin);
                                }
                                Err(err) => log::warn!("{}", err),
                            }
                        }
                        if plugins.len() == self.standalone_ui_windows.len() {
                            for (plugin, window) in
                                std::iter::zip(plugins, &mut self.standalone_ui_windows)
                            {
                                window.reload_plugins(vec![plugin]);
                            }
                        }
                    }
                }
                let _ = self.try_load_dotnet_plugin();
                if let Err(err) = self.try_load_js_plugin() {
                    log::warn!("{}", err);
                }
                if let Some(project_context) = &mut self.project_context {
                    if let Some(folder_update_type) = project_context.check_folder_notification() {
                        match folder_update_type {
                            EFolderUpdateType::Asset => {
                                let asset_folder = Self::build_asset_folder(
                                    &project_context.get_asset_folder_path(),
                                );
                                log::trace!("Update asset folder. {:?}", asset_folder);
                                self.data_source.asset_folder = Some(asset_folder.clone());
                                self.data_source.current_asset_folder = Some(asset_folder);
                                self.post_build_asset_folder();
                            }
                        }
                    }
                }
                self.player_viewport
                    .set_debug_flags(self.data_source.debug_flags);
                self.player_viewport.on_window_input(
                    rs_engine::input_type::EInputType::KeyboardInput(&self.virtual_key_code_states),
                );

                self.player_viewport
                    .set_input_mode(self.data_source.input_mode);

                self.player_viewport
                    .update_global_constants(&mut self.engine);
                self.data_source.camera_view_matrix = self.player_viewport.camera.get_view_matrix();
                self.data_source.camera_projection_matrix =
                    self.player_viewport.camera.get_projection_matrix();

                self.process_redraw_request(window_id, window, event_loop_window_target);

                update_window_with_input_mode(window, self.data_source.input_mode);
                self.engine.window_redraw_requested_end(window_id);
                window.request_redraw();
            }
            WindowEvent::Destroyed => {}
            _ => {}
        }
    }

    fn do_reload_material(
        engine: &mut rs_engine::engine::Engine,
        material_editor: &crate::material::Material,
        resolve_result: &HashMap<MaterialOptions, material_resolve::ResolveResult>,
    ) {
        if engine
            .get_settings()
            .render_setting
            .is_enable_dump_material_shader_code
        {
            if let Err(err) = Self::write_debug_shader(&material_editor, &resolve_result) {
                log::warn!("{}", err);
            }
        }
        let mut shader_code = HashMap::new();
        let mut material_info = HashMap::new();
        for (k, v) in resolve_result.iter() {
            shader_code.insert(k.clone(), v.shader_code.clone());
            material_info.insert(k.clone(), v.material_info.clone());
        }
        let handle = engine.create_material(shader_code);
        let material_content = material_editor.get_associated_material();
        let Some(material_content) = material_content else {
            return;
        };
        let mut material_content = material_content.borrow_mut();
        material_content.set_pipeline_handle(handle);
        material_content.set_material_info(material_info);
    }

    fn hotreload_material(&mut self) {
        let Some(project_context) = self.project_context.as_mut() else {
            return;
        };
        for material_editor in project_context.project.materials.clone() {
            let material_editor = material_editor.borrow();
            let snarl = &material_editor.snarl;
            let resolve_result = material_resolve::resolve(snarl, MaterialOptions::all());
            let Ok(resolve_result) = resolve_result else {
                continue;
            };
            Self::do_reload_material(&mut self.engine, &material_editor, &resolve_result);
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event<ECustomEventType>,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let mut ui_windows: Vec<&mut dyn UIWindow> = Vec::new();
        macro_rules! push_window {
            ($name:tt) => {
                if let Some(w) = self.$name.as_mut() {
                    ui_windows.push(w);
                }
            };
        }
        push_window!(material_ui_window);
        push_window!(particle_system_ui_window);
        push_window!(mesh_ui_window);
        push_window!(media_ui_window);
        push_window!(multiple_draw_ui_window);
        push_window!(blend_animation_ui_window);
        for w in &mut self.standalone_ui_windows {
            ui_windows.push(w);
        }

        match event {
            Event::DeviceEvent { event, .. } => {
                for ui_window in &mut ui_windows {
                    ui_window.on_device_event(event);
                }
                self.player_viewport.on_device_event(event);
            }
            Event::UserEvent(event) => {
                let window = self.window_manager.borrow_mut().get_main_window();
                self.process_custom_event(event, &mut *window.borrow_mut());
            }
            Event::WindowEvent { event, window_id } => {
                let window_id = u64::from(*window_id) as isize;
                let Some(window_type) = self
                    .window_manager
                    .borrow_mut()
                    .get_window_type_by_id(window_id)
                else {
                    return;
                };
                let Some(window) = self.window_manager.borrow_mut().get_window_by_id(window_id)
                else {
                    return;
                };

                if matches!(event, WindowEvent::RedrawRequested)
                    && window_type == EWindowType::Standalone
                {
                    let other_ui_windows = ui_windows
                        .iter_mut()
                        .filter(|x| x.get_window_id() != window_id);
                    for other_ui_window in other_ui_windows {
                        let mut _window_manager = self.window_manager.borrow_mut();
                        let id = other_ui_window.get_window_id();
                        if let Some(window) = _window_manager.get_window_by_id(id) {
                            let mut window = window.borrow_mut();
                            let mut is_request_close = false;
                            if window.has_focus() == false {
                                other_ui_window.on_window_event(
                                    id,
                                    &mut window,
                                    event,
                                    event_loop_window_target,
                                    &mut self.engine,
                                    &mut _window_manager,
                                    &mut is_request_close,
                                );
                            }
                        }
                    }
                }

                let window = &mut *window.borrow_mut();
                let egui_event_response = self.egui_winit_state.on_window_event(window, event);

                let mut close_windows = vec![];

                for ui_window in &mut ui_windows {
                    if ui_window.get_window_id() != window_id {
                        continue;
                    }
                    if let WindowEvent::Resized(window_size) = &event {
                        self.engine
                            .resize(window_id, window_size.width, window_size.height);
                    };
                    let mut is_request_close = false;
                    let mut is_close = false;
                    ui_window.on_window_event(
                        window_id,
                        window,
                        event,
                        event_loop_window_target,
                        &mut self.engine,
                        &mut self.window_manager.borrow_mut(),
                        &mut is_request_close,
                    );
                    if is_request_close {
                        is_close = true;
                    }
                    if let WindowEvent::CloseRequested = &event {
                        is_close = true;
                    };
                    if is_close {
                        self.window_manager
                            .borrow_mut()
                            .remove_window_by_id(&window_id);
                        self.engine.remove_window(window_id);
                        close_windows.push(window_id);
                    }
                }

                macro_rules! take_window {
                    ($name:tt) => {
                        if let Some(w) = self.$name.as_mut() {
                            if close_windows.contains(&w.get_window_id()) {
                                self.$name = None;
                            }
                        }
                    };
                }
                take_window!(material_ui_window);
                take_window!(particle_system_ui_window);
                take_window!(mesh_ui_window);
                take_window!(media_ui_window);
                take_window!(multiple_draw_ui_window);
                take_window!(blend_animation_ui_window);

                self.standalone_ui_windows
                    .retain(|x| !close_windows.contains(&x.get_window_id()));

                if let Some(Some(event)) = self
                    .material_ui_window
                    .as_mut()
                    .map(|x| &mut x.material_view.event)
                {
                    match event {
                        material_view::EEventType::Update(material, resolve_result) => {
                            Self::do_reload_material(
                                &mut self.engine,
                                &material.borrow(),
                                resolve_result,
                            );
                        }
                    }
                }

                match window_type {
                    EWindowType::Main => self.main_window_event_process(
                        window_id,
                        window,
                        event,
                        event_loop_window_target,
                        egui_event_response,
                    ),
                    _ => {}
                }
            }
            Event::NewEvents(_) => {}
            Event::LoopExiting => {}
            _ => {}
        }
    }

    // fn try_load_plugin(&mut self) -> anyhow::Result<()> {
    //     let plugin = self.try_create_plugin()?;
    //     self.plugins.push(plugin);
    //     Ok(())
    // }

    #[cfg(feature = "plugin_shared_crate")]
    fn try_create_plugin(&mut self) -> anyhow::Result<Box<dyn Plugin>> {
        if let Some(project_context) = self.project_context.as_mut() {
            project_context.reload()?;
            let lib = project_context.hot_reload.get_library_reload();
            let lib = lib.lock().unwrap();
            let func = lib.load_symbol::<rs_engine::plugin::signature::CreatePlugin>(
                rs_engine::plugin::symbol_name::CREATE_PLUGIN,
            )?;
            let plugin = func();
            return Ok(plugin);
        }
        return Err(anyhow!("Can not create plugin"));
    }

    fn try_load_dotnet_plugin(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "plugin_dotnet")]
        if let Some(project_context) = self.project_context.as_mut() {
            if let Some(donet_host) = self.donet_host.as_mut() {
                if !donet_host.is_watching() {
                    let file_path = project_context.get_dotnet_script_shared_lib_path();
                    let folder = file_path.parent().ok_or(anyhow!("No parent folder"))?;
                    let file_name = file_path
                        .file_name()
                        .map(|x| x.to_str())
                        .flatten()
                        .ok_or(anyhow!("No file name"))?;
                    donet_host.start_watch(folder, file_name)?;
                    donet_host.reload_script()?;
                }
                if donet_host.is_need_reload() {
                    donet_host.reload_script()?;
                }
            }
        }
        Ok(())
    }

    fn try_load_js_plugin(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "plugin_v8")]
        if let Some(project_context) = self.project_context.as_mut() {
            if let Some(v8_runtime) = self.v8_plugin.as_mut().map(|x| &mut x.runtime) {
                if !v8_runtime.is_watching() {
                    let entry_path = project_context.get_js_script_entry_path();
                    let root_dir = project_context.get_js_script_root_dir();
                    v8_runtime.start_watch(root_dir, entry_path)?;
                    v8_runtime.reload_script()?;
                }
                if v8_runtime.is_need_reload() {
                    v8_runtime.reload_script()?;
                }
            }
        }
        Ok(())
    }
    fn post_build_asset_folder(&mut self) {
        let Some(project_context) = &self.project_context else {
            return;
        };
        let Some(asset_folder) = &self.data_source.asset_folder else {
            return;
        };
        self.editor_ui
            .content_item_property_view
            .image_asset_files
            .clear();
        let base = project_context.get_project_folder_path();
        for file in &asset_folder.files {
            match file.get_file_type() {
                EFileType::Exr | EFileType::Hdr => {
                    if let Ok(path) = file.path.strip_prefix(base.clone()) {
                        self.editor_ui
                            .content_item_property_view
                            .image_asset_files
                            .push(path.to_path_buf());
                    }
                }
                _ => {}
            }
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
            let Ok(entry) = entry else {
                continue;
            };
            if entry.path() == path {
                continue;
            }
            if entry.path().is_dir() {
                let path = entry.path();
                let sub_folder = Self::build_asset_folder(path);
                folder.folders.push(sub_folder);
            } else {
                let Some(extension) = entry.path().extension() else {
                    continue;
                };
                let extension = extension.to_ascii_lowercase();
                let extension = extension.to_str().unwrap();
                let Some(file_type) = EFileType::from_str(extension) else {
                    continue;
                };
                if !SUPPORT_ASSET_FILE_TYPES.contains(&file_type) {
                    continue;
                }
                let asset_file = AssetFile {
                    name: entry.file_name().to_str().unwrap().to_string(),
                    path: entry.path().to_path_buf(),
                };
                folder.files.push(asset_file);
            }
        }
        folder
    }

    fn process_keyboard_input(
        &mut self,
        _: &winit::event::DeviceId,
        event: &winit::event::KeyEvent,
        _: bool,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
            return;
        };

        self.virtual_key_code_states
            .insert(virtual_keycode, event.state);

        if Self::is_keys_pressed(
            &mut self.virtual_key_code_states,
            &[KeyCode::Backquote],
            true,
        ) {
            self.data_source.is_console_cmds_view_open =
                !self.data_source.is_console_cmds_view_open;
        }

        if self.data_source.input_mode == EInputMode::UI {
            if Self::is_keys_pressed(&mut self.virtual_key_code_states, &[KeyCode::KeyR], true) {
                self.editor_ui.gizmo_view.gizmo_mode = GizmoMode::all_scale();
            }
            if Self::is_keys_pressed(&mut self.virtual_key_code_states, &[KeyCode::KeyW], true) {
                self.editor_ui.gizmo_view.gizmo_mode = GizmoMode::all_translate();
            }
            if Self::is_keys_pressed(&mut self.virtual_key_code_states, &[KeyCode::KeyE], true) {
                self.editor_ui.gizmo_view.gizmo_mode = GizmoMode::all_rotate();
            }
            if Self::is_keys_pressed(&mut self.virtual_key_code_states, &[KeyCode::Space], true) {
                let old_gizmo_orientation = &mut self.editor_ui.gizmo_view.gizmo_orientation;
                match old_gizmo_orientation {
                    GizmoOrientation::Global => {
                        *old_gizmo_orientation = GizmoOrientation::Local;
                    }
                    GizmoOrientation::Local => {
                        *old_gizmo_orientation = GizmoOrientation::Global;
                    }
                }
            }
        }

        let shading_types = HashMap::from([
            (
                KeyCode::Digit1,
                rs_render::global_uniform::EDebugShadingType::None,
            ),
            (
                KeyCode::Digit2,
                rs_render::global_uniform::EDebugShadingType::BaseColor,
            ),
            (
                KeyCode::Digit3,
                rs_render::global_uniform::EDebugShadingType::Metallic,
            ),
            (
                KeyCode::Digit4,
                rs_render::global_uniform::EDebugShadingType::Roughness,
            ),
            (
                KeyCode::Digit5,
                rs_render::global_uniform::EDebugShadingType::Normal,
            ),
            (
                KeyCode::Digit6,
                rs_render::global_uniform::EDebugShadingType::VertexColor0,
            ),
            (
                KeyCode::Digit7,
                rs_render::global_uniform::EDebugShadingType::Shadow,
            ),
        ]);
        for (key_code, debug_shading_type) in shading_types {
            if Self::is_keys_pressed(
                &mut self.virtual_key_code_states,
                &[KeyCode::AltLeft, key_code],
                true,
            ) {
                self.data_source.debug_shading_type = debug_shading_type;
                self.player_viewport
                    .set_debug_shading(self.data_source.debug_shading_type);
            }
        }

        if Self::is_keys_pressed(
            &mut self.virtual_key_code_states,
            &[KeyCode::AltLeft, KeyCode::F4],
            true,
        ) {
            event_loop_window_target.exit();
        }

        if Self::is_keys_pressed(
            &mut self.virtual_key_code_states,
            &[KeyCode::ControlLeft, KeyCode::KeyG],
            false,
        ) {
            self.player_viewport.toggle_grid_visible();
            let is_show_debug = !self.data_source.is_show_debug;
            if let Some(level) = &mut self.data_source.level {
                let mut level = level.borrow_mut();
                if is_show_debug {
                    level.set_debug_show_flag(rs_engine::debug_show_flag::DebugShowFlag::all());
                } else {
                    level.set_debug_show_flag(rs_engine::debug_show_flag::DebugShowFlag::empty());
                }
            }
            self.data_source.is_show_debug = is_show_debug;
        }

        if Self::is_keys_pressed(&mut self.virtual_key_code_states, &[KeyCode::Escape], true) {
            self.editor_ui.object_property_view.selected_object = None;
        }

        if Self::is_keys_pressed(
            &mut self.virtual_key_code_states,
            &[KeyCode::ControlLeft, KeyCode::KeyS],
            true,
        ) {
            self.save_current_project();
        }

        if Self::is_keys_pressed(&mut self.virtual_key_code_states, &[KeyCode::F5], true) {
            self.open_standalone_window(event_loop_window_target, StandaloneSimulationType::Single);
        }
    }

    fn is_keys_pressed(
        virtual_key_code_states: &mut HashMap<KeyCode, ElementState>,
        keys: &[KeyCode],
        is_consume: bool,
    ) -> bool {
        let mut states: HashMap<KeyCode, ElementState> = HashMap::new();
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
            true
        } else {
            false
        }
    }

    fn is_project_name_valid(name: &str) -> bool {
        if name.is_empty() || name.len() > 127 {
            return false;
        }
        let reg = regex::Regex::new("^[a-zA-Z]*$").unwrap();
        reg.is_match(name)
    }

    fn load_ibl_content_resource(
        engine: &mut rs_engine::engine::Engine,
        project_context: &ProjectContext,
        ibl: SingleThreadMutType<rs_engine::content::ibl::IBL>,
    ) -> anyhow::Result<()> {
        let url = ibl.borrow().url.clone();
        let image_reference = &ibl.borrow().image_reference;
        let Some(image_reference) = image_reference.as_ref() else {
            return Ok(());
        };
        let file_path = project_context
            .get_project_folder_path()
            .join(image_reference);
        if !file_path.exists() {
            return Err(anyhow!("The file is not exist"));
        }
        if project_context
            .get_ibl_bake_cache_dir(image_reference)
            .exists()
        {
            let name = rs_engine::url_extension::UrlExtension::get_name_in_editor(&url);
            let ibl_baking = rs_artifact::ibl_baking::IBLBaking {
                name,
                url: url.clone(),
                brdf_data: std::fs::read(
                    project_context
                        .get_ibl_bake_cache_dir(image_reference)
                        .join("brdf.dds"),
                )?,
                pre_filter_data: std::fs::read(
                    project_context
                        .get_ibl_bake_cache_dir(image_reference)
                        .join("pre_filter.dds"),
                )?,
                irradiance_data: std::fs::read(
                    project_context
                        .get_ibl_bake_cache_dir(image_reference)
                        .join("irradiance.dds"),
                )?,
            };
            log::trace!("Load IBL {}, use bake ibl", url.to_string());
            engine.upload_prebake_ibl(url.clone(), ibl_baking);
            return Ok(());
        }
        let save_dir = project_context.try_create_ibl_bake_cache_dir(image_reference)?;
        log::trace!("Load IBL {}, bake ibl", url.to_string());
        engine.ibl_bake(
            &file_path,
            url,
            ibl.borrow().bake_info.clone(),
            Some(&save_dir),
        );
        Ok(())
    }

    pub fn write_debug_shader(
        material_editor: &crate::material::Material,
        resolve_result: &HashMap<MaterialOptions, material_resolve::ResolveResult>,
    ) -> anyhow::Result<()> {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct DebugInfo {
            name: String,
            material_options: MaterialOptions,
        }
        let name = material_editor.url.get_name_in_editor();
        for (option, result) in resolve_result.clone() {
            let mut hasher = std::hash::DefaultHasher::new();
            std::hash::Hash::hash(&option, &mut hasher);
            let output_file_name = format!("{}_{}.wgsl", name, std::hash::Hasher::finish(&hasher));
            let output_folder = file_manager::get_current_exe_dir()?.join("debug_shader");
            std::fs::create_dir_all(&output_folder)?;
            let output_file = output_folder.join(output_file_name);
            let debug_info = DebugInfo {
                name: name.clone(),
                material_options: option,
            };
            let string = serde_json::to_string_pretty(&debug_info)?;
            let lines = string.split("\n");
            let mut contents = String::new();
            for line in lines {
                contents.push_str(&format!("// {}\n", line));
            }
            contents.push_str("\n");
            contents.push_str(&result.shader_code);
            if output_file.exists() {
                std::fs::remove_file(&output_file)?;
            }
            std::fs::write(output_file, contents)?;
        }
        Ok(())
    }

    fn create_multi_res_mesh_cache_non_blocking(
        project_context: &ProjectContext,
        static_mesh: &rs_engine::content::static_mesh::StaticMesh,
    ) -> anyhow::Result<()> {
        if !static_mesh.is_enable_multiresolution {
            return Ok(());
        }
        rs_core_minimal::thread_pool::ThreadPool::global().spawn({
            let mesh_cluster_dir = project_context.try_create_mesh_cluster_dir()?;
            let static_mesh_artiface_url = static_mesh.asset_info.get_url();
            move || match Self::create_multi_res_mesh_cache(
                &mesh_cluster_dir,
                static_mesh_artiface_url,
            ) {
                Ok(_) => {}
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
        });
        Ok(())
    }

    fn create_multi_res_mesh_cache(
        mesh_cluster_dir: &Path,
        static_mesh_artiface_url: url::Url,
    ) -> anyhow::Result<ClusterCollection> {
        let rm = ResourceManager::default();
        let static_mesh_result = rm.get_static_mesh(&static_mesh_artiface_url)?;
        let indices = &static_mesh_result.indexes;

        let mut vertices: Vec<VertexPosition> =
            Vec::with_capacity(static_mesh_result.vertexes.len());
        for item in static_mesh_result.vertexes.iter() {
            vertices.push(VertexPosition::new(item.position));
        }
        let vertices = Arc::new(vertices);
        let gpmetis_program_path: Option<std::path::PathBuf> = None;

        let cluster_collection = ClusterCollection::parallel_from_indexed_vertices(
            indices,
            vertices,
            gpmetis_program_path,
        )?;

        let filename = static_mesh_result.name.clone();
        let output_path = mesh_cluster_dir.join(filename);
        let data = rs_artifact::bincode_legacy::serialize(&cluster_collection, None)?;
        let _ = std::fs::write(output_path, data)?;
        Ok(cluster_collection)
    }

    fn content_load_resources(
        engine: &mut rs_engine::engine::Engine,
        model_loader: &mut ModelLoader,
        project_context: &ProjectContext,
        files: Vec<EContentFileType>,
    ) {
        let result = crate::load_content::load_contents::LoadContents::load(
            engine,
            project_context,
            model_loader,
            &files,
        );
        if let Err(err) = result {
            log::warn!("{}", err);
        }
    }

    fn add_new_actors(
        level: &mut rs_engine::content::level::Level,
        engine: &mut rs_engine::engine::Engine,
        actors: Vec<Rc<RefCell<rs_engine::actor::Actor>>>,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        level.add_new_actors(engine, actors, files, player_viewport);
    }

    fn open_project(
        &mut self,
        file_path: &Path,
        window: &winit::window::Window,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!();
        let project_context = ProjectContext::open(&file_path)?;
        file_manager::set_current_project_dir(&project_context.get_project_folder_path());
        self.engine
            .get_logger_mut()
            .add_white_list(project_context.project.project_name.clone());
        window.set_title(&format!("Editor({})", project_context.project.project_name));
        let asset_folder_path = project_context.get_asset_folder_path();
        let asset_folder = Self::build_asset_folder(&asset_folder_path);
        self.editor_ui
            .set_project_folder_path(Some(project_context.get_project_folder_path()));
        log::trace!("Update asset folder. {:?}", asset_folder);
        self.data_source.asset_folder = Some(asset_folder.clone());
        self.data_source.current_asset_folder = Some(asset_folder);

        self.data_source.content_data_source.current_folder =
            Some(project_context.project.content.clone());
        Self::content_load_resources(
            &mut self.engine,
            &mut self.model_loader,
            &project_context,
            project_context.project.content.borrow().files.clone(),
        );

        self.engine
            .on_content_files_changed(project_context.project.content.borrow().files_to_map(true));

        self.engine
            .set_settings(project_context.project.settings.borrow().clone());
        self.data_source.project_settings = Some(project_context.project.settings.clone());

        {
            let binding = project_context.project.content.borrow();
            let find_level = binding
                .files
                .iter()
                .find(|x| match x {
                    EContentFileType::Level(_) => true,
                    _ => false,
                })
                .map(|x| match x {
                    EContentFileType::Level(level) => Some(level),
                    _ => None,
                })
                .flatten();

            self.data_source.level = find_level.cloned();

            if let Some(level) = find_level.cloned() {
                let mut level = level.borrow_mut();
                if let Some(folder) = &self.data_source.content_data_source.current_folder {
                    level.initialize(
                        &mut self.engine,
                        &folder.borrow().files,
                        &mut self.player_viewport,
                    );
                }
            }
        }

        {
            let mut materials = self.editor_ui.object_property_view.materials.borrow_mut();
            materials.clear();
            let mut animations = self.editor_ui.object_property_view.animations.borrow_mut();
            animations.clear();
            let mut static_meshes = self
                .editor_ui
                .object_property_view
                .static_meshes
                .borrow_mut();
            static_meshes.clear();

            let files = &project_context.project.content.borrow().files;
            for file in files {
                match file {
                    EContentFileType::Material(material) => {
                        let url = material.borrow().url.clone();
                        materials.push(url);
                    }
                    EContentFileType::SkeletonAnimation(animation) => {
                        let url = animation.borrow().url.clone();
                        animations.push(url);
                    }
                    EContentFileType::BlendAnimations(animation) => {
                        let url = animation.borrow().url.clone();
                        animations.push(url);
                    }
                    EContentFileType::StaticMesh(static_mesh) => {
                        let url = static_mesh.borrow().url.clone();
                        static_meshes.push(url);
                    }
                    _ => {}
                }
            }
        }

        let rm = ResourceManager::default();
        self.editor_ui.debug_textures_view.all_texture_urls = rm.get_texture_urls();

        self.project_context = Some(project_context);
        // log::trace!("{:?}", self.try_load_plugin());
        let _ = self.try_load_dotnet_plugin();
        if let Err(err) = self.try_load_js_plugin() {
            log::warn!("{}", err);
        }
        self.data_source
            .recent_projects
            .paths
            .insert(0, file_path.to_path_buf());
        self.data_source.recent_projects.save()?;
        self.post_build_asset_folder();

        Ok(())
    }

    fn process_custom_event(
        &mut self,
        event: &ECustomEventType,
        window: &mut winit::window::Window,
    ) {
        match event {
            ECustomEventType::OpenFileDialog(dialog_type) => match dialog_type {
                EFileDialogType::NewProject(name) => {
                    let result = (|| {
                        if !Self::is_project_name_valid(&name) {
                            return Err(anyhow!("Not a valid project name"));
                        }
                        let dialog = rfd::FileDialog::new();
                        let folder = dialog.pick_folder().ok_or(anyhow!("Fail to pick folder"))?;
                        log::trace!("Selected folder: {:?}", folder);
                        let project_file_path = Project::create_empty_project(&folder, name)?;
                        self.open_project(&project_file_path, window)?;
                        self.data_source.is_new_project_window_open = false;
                        Ok(())
                    })();
                    log::trace!("{:?}", result);
                }
                EFileDialogType::OpenProject => {
                    let dialog = rfd::FileDialog::new().add_filter("Project", &["rsproject"]);
                    let Some(file_path) = dialog.pick_file() else {
                        return;
                    };
                    log::trace!("Selected file: {:?}", file_path);
                    let result = self.open_project(&file_path, window);
                    log::trace!("{:?}", result);
                }
            },
        }
    }

    fn open_project_workspace(file_path: &Path) -> anyhow::Result<()> {
        // https://github.com/rust-lang/rust/issues/37519#issuecomment-1694489623
        let arg = file_path
            .to_str()
            .ok_or(anyhow!("Not a valid path, {file_path:?}"))?;
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "Code"])
                .arg(arg)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        } else {
            Command::new("sh")
                .args(["-c", "Code"])
                .arg(arg)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        };
        match result {
            Ok(child) => match child.stderr {
                Some(stderr) => Err(anyhow!("{stderr:?}")),
                None => Ok(()),
            },
            Err(err) => Err(err.into()),
        }
    }

    fn open_model_file(&mut self, file_path: PathBuf) -> anyhow::Result<()> {
        let exist_names = self.get_all_content_names();
        let project_context = self
            .project_context
            .as_mut()
            .ok_or(anyhow!("Project context is null"))?;
        let active_level = self
            .data_source
            .level
            .clone()
            .ok_or(anyhow!("Active level is null"))?;
        let file_type = EFileType::from_path(&file_path)
            .context(format!("Invalid file type. {:?}", file_path))?;
        if !SUPPORT_ASSET_MODEL_FILE_TYPES.contains(&file_type) {
            return Err(anyhow!("Not support file type. {:?}", file_type));
        }
        let asset_reference = file_path
            .strip_prefix(project_context.get_asset_folder_path())?
            .to_str()
            .ok_or(anyhow!("Incorrect path: {:?}", file_path))?;

        let actor_names: Vec<String> = {
            active_level
                .borrow()
                .actors
                .iter()
                .map(|x| x.borrow().name.clone())
                .collect()
        };
        let load_result = self.model_loader.load_from_file_as_actor(
            &file_path,
            asset_reference.to_string(),
            exist_names,
            actor_names,
        )?;

        let mut add_files: Vec<EContentFileType> = vec![];
        for static_mesh in &load_result.static_meshes {
            add_files.push(EContentFileType::StaticMesh(static_mesh.clone()));
        }
        for skeleton_meshe in &load_result.skeleton_meshes {
            add_files.push(EContentFileType::SkeletonMesh(skeleton_meshe.clone()));
        }
        for node_animation in &load_result.node_animations {
            self.editor_ui
                .object_property_view
                .animations
                .borrow_mut()
                .push(node_animation.borrow().url.clone());
            add_files.push(EContentFileType::SkeletonAnimation(node_animation.clone()));
        }
        if let Some(skeleton) = &load_result.skeleton {
            add_files.push(EContentFileType::Skeleton(skeleton.clone()));
        }
        let content = project_context.project.content.clone();
        let mut content = content.borrow_mut();
        Self::content_load_resources(
            &mut self.engine,
            &mut self.model_loader,
            project_context,
            add_files.clone(),
        );
        content.files.append(&mut add_files);
        let mut active_level = active_level.borrow_mut();
        let new_actor = SingleThreadMut::new(rs_engine::actor::Actor::new_with_node(
            load_result.appropriate_name,
            load_result.scene_node,
        ));
        Self::add_new_actors(
            &mut active_level,
            &mut self.engine,
            vec![new_actor],
            &content.files,
            &mut self.player_viewport,
        );
        // active_level.init_actor_physics(load_result.actor.clone());
        // active_level.actors.push(load_result.actor);

        // let mesh_clusters = ModelLoader::load_from_file(&file_path, &[])?;
        // self.data_source.is_model_hierarchy_open = true;
        // let mut items: Vec<Rc<MeshItem>> = vec![];
        // for mesh_cluster in mesh_clusters {
        //     let item = MeshItem {
        //         name: mesh_cluster.name,
        //         childs: vec![],
        //     };
        //     items.push(Rc::new(item));
        // }
        // let model_view_data = ModelViewData {
        //     mesh_items: items,
        //     file_path,
        // };
        // self.data_source.model_view_data = Some(model_view_data);
        Ok(())
    }

    #[cfg(feature = "plugin_v8")]
    fn process_v8_plugin_tick(&mut self) -> anyhow::Result<()> {
        if let (Some(v8_plugin), Some(level)) =
            (self.v8_plugin.as_mut(), self.data_source.level.as_mut())
        {
            let v8_runtime = &mut v8_plugin.runtime;
            let v8_register_manager = &mut v8_plugin.binding_api_manager;
            let wrapped_engine = v8_register_manager.engine_api.get_wrapped_value();
            let wrapped_level = v8_register_manager
                .level_api
                .make_wrapped_value(v8_runtime, level.clone())?;

            let wrapped_player_viewport = v8_register_manager
                .player_viewport_binding_api
                .get_wrapped_value();

            let result = v8_runtime
                .tick(wrapped_engine, wrapped_level, wrapped_player_viewport)
                .map_err(|err| anyhow!("{err}"));
            return result;
        }
        Ok(())
    }

    fn process_redraw_request(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        if let Some(active_level) = self.data_source.level.clone() {
            let mut active_level = active_level.borrow_mut();
            active_level.set_physics_simulate(self.data_source.is_simulate_real_time);
            active_level.tick(
                self.engine.get_game_time(),
                &mut self.engine,
                &mut self.player_viewport,
            );

            let mut draw_objects = active_level.collect_draw_objects();

            for camera_componenet in active_level.collect_camera_componenets() {
                let camera_componenet = camera_componenet.borrow();
                if let Some(player_viewport) = camera_componenet.get_player_viewport() {
                    let mut player_viewport = player_viewport.borrow_mut();
                    player_viewport.update_global_constants(&mut self.engine);
                    for draw_object in draw_objects.iter_mut() {
                        player_viewport.update_draw_object(&mut self.engine, draw_object);
                        draw_object.switch_player_viewport(&player_viewport);
                    }
                    player_viewport.append_to_draw_list(&draw_objects);
                    self.engine.present_player_viewport(&mut player_viewport);
                }
            }

            for draw_object in draw_objects.iter_mut() {
                self.player_viewport
                    .update_draw_object(&mut self.engine, draw_object);
                draw_object.switch_player_viewport(&self.player_viewport);
            }
            self.player_viewport.append_to_draw_list(&draw_objects);

            if let Some(physics) = active_level.get_physics_mut() {
                self.player_viewport.physics_debug(
                    &mut self.engine,
                    &physics.rigid_body_set,
                    &physics.collider_set,
                );
            }
        }

        crate::ui::misc::ui_begin(&mut self.egui_winit_state, window);

        self.process_ui_event(window, event_loop_window_target);

        #[cfg(feature = "plugin_dotnet")]
        if let Some(dotnet) = self.donet_host.as_mut() {
            dotnet.application.tick(&mut self.engine);
        }

        #[cfg(feature = "plugin_v8")]
        if let Err(err) = self.process_v8_plugin_tick() {
            log::warn!("{err}");
        }

        if let Some(ui_window) = &mut self.particle_system_ui_window {
            ui_window
                .base_ui_window
                .egui_winit_state
                .egui_ctx()
                .show_viewport_deferred(
                    ui_window
                        .base_ui_window
                        .egui_winit_state
                        .egui_input()
                        .viewport_id,
                    egui::ViewportBuilder::default(),
                    |_, _| {},
                );
        }

        let gui_render_output =
            crate::ui::misc::ui_end(&mut self.egui_winit_state, window, window_id);

        self.engine.send_render_task({
            move |renderer| {
                let rm = ResourceManager::default();
                let visualization_url =
                    build_built_in_resouce_url("ShadowDepthTextureVisualization").unwrap();
                let shadow_depth_texture_url =
                    build_built_in_resouce_url("PlayerViewport.ShadowDepthTexture").unwrap();
                let Some(texture_handle) = rm.get_texture_by_url(&visualization_url) else {
                    return;
                };
                let Some(shadow_texture_handle) = rm.get_texture_by_url(&shadow_depth_texture_url)
                else {
                    return;
                };
                let texture_handle = *texture_handle;
                let shadow_texture_handle = *shadow_texture_handle;
                let Some(input_texture) = renderer.get_textures(shadow_texture_handle) else {
                    return;
                };
                let Some(output_texture) = renderer.get_textures(texture_handle) else {
                    return;
                };

                let input_texture_view =
                    input_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let output_texture_view =
                    output_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let device = renderer.get_device();
                let queue = renderer.get_queue();
                let shader_library = renderer.get_shader_library();
                let pool = renderer.get_base_compute_pipeline_pool();
                let pipeline = rs_render::compute_pipeline::format_conversion::Depth32FloatConvertRGBA8UnormPipeline::new(device, shader_library, &pool);
                pipeline.execute(device, queue, &input_texture_view, &output_texture_view, glam::uvec3(1024, 1024, 1));
            }
        });
        self.engine
            .present_player_viewport(&mut self.player_viewport);
        self.engine.draw_gui(gui_render_output);
    }

    pub fn prepreocess_shader() -> anyhow::Result<()> {
        let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();
        let output_path =
            rs_core_minimal::file_manager::get_engine_output_target_dir().join("shaders");
        if !output_path.exists() {
            std::fs::create_dir(output_path.clone())
                .context(anyhow!("Can not create dir {:?}", output_path))?;
        }

        let mut compile_commands = vec![];
        for buildin_shader in buildin_shaders {
            let description = buildin_shader.get_shader_description();
            let name = buildin_shader.get_name();
            let processed_code = rs_shader_compiler_core::pre_process::pre_process(
                &description.shader_path,
                description.include_dirs.iter(),
                description.definitions.iter(),
            )?;
            let filepath = output_path.join(name);
            std::fs::write(filepath.clone(), processed_code)
                .context(anyhow!("Can not write to file {:?}", filepath))?;

            let compile_command = buildin_shader.as_ref().to_compile_command();
            compile_commands.push(compile_command);
        }
        let output_path = rs_core_minimal::file_manager::get_engine_root_dir().join(".vscode");
        if !output_path.exists() {
            std::fs::create_dir(output_path.clone())
                .context(anyhow!("Can not create dir {:?}", output_path))?;
        }
        let target_path = output_path.join("shader_compile_commands.json");
        let _ = std::fs::write(
            target_path.clone(),
            serde_json::to_string(&compile_commands)?,
        )
        .context(anyhow!("Can not write to file {:?}", target_path))?;
        Ok(())
    }

    fn create_standalone_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        standalone_simulation_type: StandaloneSimulationType,
    ) -> Option<StandaloneUiWindow> {
        let Some(level) = self.data_source.level.clone() else {
            return None;
        };
        let active_level = &level.borrow();
        #[cfg(feature = "plugin_shared_crate")]
        let plugins = {
            let dynamic_plugins = self.try_create_plugin().map_or(vec![], |x| vec![x]);
            let static_plugins = rs_proc_macros::load_static_plugins!(rs_editor);
            if static_plugins.is_empty() {
                log::trace!("Using dynamic plugins");
                dynamic_plugins
            } else {
                log::trace!("Using static plugins");
                static_plugins
            }
        };
        let contents = self.get_all_contents();
        let ui_window = StandaloneUiWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
            #[cfg(feature = "plugin_shared_crate")]
            plugins,
            active_level,
            contents,
            standalone_simulation_type,
        )
        .expect("Should be opened");
        return Some(ui_window);
    }

    fn open_standalone_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        standalone_simulation_type: StandaloneSimulationType,
    ) {
        assert!(self.standalone_ui_windows.is_empty());
        match standalone_simulation_type {
            StandaloneSimulationType::Single => {
                let standalone_window = self
                    .create_standalone_window(event_loop_window_target, standalone_simulation_type);
                if let Some(standalone_window) = standalone_window {
                    self.standalone_ui_windows.push(standalone_window);
                }
            }
            StandaloneSimulationType::MultiplePlayer(multiple_player_options) => {
                assert_eq!(
                    self.data_source.multiple_players,
                    multiple_player_options.players
                );
                for i in 0..self.data_source.multiple_players {
                    let is_server = i == 0;
                    let options = MultiplePlayerOptions {
                        server_socket_addr: multiple_player_options.server_socket_addr,
                        is_server,
                        players: multiple_player_options.players,
                    };
                    let standalone_window = self.create_standalone_window(
                        event_loop_window_target,
                        StandaloneSimulationType::MultiplePlayer(options),
                    );
                    if let Some(standalone_window) = standalone_window {
                        self.standalone_ui_windows.push(standalone_window);
                    }
                }
            }
        }
    }

    fn open_particle_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        particle_system: Rc<RefCell<rs_engine::content::particle_system::ParticleSystem>>,
    ) {
        let ui_window = ParticleSystemUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
            particle_system,
        )
        .expect("Should be opened");
        self.particle_system_ui_window = Some(ui_window);
    }

    fn open_material_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        open_material: Option<Rc<RefCell<rs_engine::content::material::Material>>>,
    ) {
        let Some(project_context) = &mut self.project_context else {
            return;
        };
        let folder = project_context.project.content.clone();
        let mut ui_window = MaterialUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
            folder,
        )
        .expect("Should be opened");
        if let Some(open_material) = open_material {
            let url = &open_material.borrow().asset_url;
            if let Some(project_context) = &self.project_context {
                if let Some(asset) = project_context
                    .project
                    .materials
                    .iter()
                    .find(|x| &x.borrow().url == url)
                {
                    asset
                        .borrow_mut()
                        .set_associated_material(open_material.clone());
                    ui_window.data_source.current_open_material = Some(asset.clone());
                };
            }
        }
        ui_window.material_view.viewer.texture_urls = self.collect_textures();
        ui_window.material_view.viewer.virtual_texture_urls = self.collect_virtual_textures();
        ui_window.material_view.viewer.is_updated = true;
        self.material_ui_window = Some(ui_window);
    }

    fn open_skin_mesh_window(
        &mut self,
        skeleton_mesh: &mut rs_engine::content::skeleton_mesh::SkeletonMesh,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let Some(project_context) = &self.project_context else {
            return;
        };
        let project_folder_path = project_context.get_project_folder_path();
        let mut ui_window = MeshUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
        )
        .expect("Should be opened");
        let file_path = project_folder_path.join(&skeleton_mesh.get_relative_path());
        self.model_loader
            .load_scene_from_file_and_cache(&file_path)
            .unwrap();
        let skin_mesh = self.model_loader.to_runtime_cache_skin_mesh(
            skeleton_mesh,
            &project_folder_path,
            ResourceManager::default(),
        );
        ui_window.update(&mut self.engine, &skin_mesh.vertexes, &skin_mesh.indexes);
        self.mesh_ui_window = Some(ui_window);
    }

    fn open_static_mesh_window(
        &mut self,
        static_mesh: &mut rs_engine::content::static_mesh::StaticMesh,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let Some(project_context) = &self.project_context else {
            return;
        };
        let asset_folder_path = project_context.get_asset_folder_path();
        let mut ui_window = MeshUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
        )
        .expect("Should be opened");
        let file_path = asset_folder_path.join(&static_mesh.asset_info.relative_path);
        self.model_loader
            .load_scene_from_file_and_cache(&file_path)
            .unwrap();
        let Ok(static_mesh) = self.model_loader.to_runtime_cache_static_mesh(
            static_mesh,
            &asset_folder_path,
            ResourceManager::default(),
        ) else {
            return;
        };
        ui_window.update2(
            &mut self.engine,
            &static_mesh.vertexes,
            &static_mesh.indexes,
        );
        self.mesh_ui_window = Some(ui_window);
    }

    fn open_media_window(
        &mut self,
        media_path: impl AsRef<Path>,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let mut ui_window = MediaUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
        )
        .expect("Should be opened");
        let result = ui_window.update(media_path);
        log::trace!("{:?}", result);
        self.media_ui_window = Some(ui_window);
    }

    fn open_multiple_draw_ui_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let mut ui_window = MultipleDrawUiWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
        )
        .expect("Should be opened");
        ui_window.update(&mut self.engine);
        self.multiple_draw_ui_window = Some(ui_window);
    }

    fn open_blend_animation_ui_window(
        &mut self,
        blend_animation: SingleThreadMutType<BlendAnimations>,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let Some(project_context) = self.project_context.as_ref() else {
            return;
        };
        let content = project_context.project.content.clone();
        let ui_window = BlendAnimationUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
            content,
            blend_animation,
        )
        .expect("Should be opened");
        self.blend_animation_ui_window = Some(ui_window);
    }

    fn collect_textures(&self) -> Vec<url::Url> {
        let Some(project_context) = &self.project_context else {
            return vec![];
        };
        project_context
            .project
            .content
            .borrow()
            .files
            .iter()
            .filter_map(|x| match x {
                EContentFileType::Texture(_) => Some(x.get_url()),
                _ => None,
            })
            .collect()
    }

    fn collect_virtual_textures(&self) -> Vec<url::Url> {
        let Some(project_context) = &self.project_context else {
            return vec![];
        };
        project_context
            .project
            .content
            .borrow()
            .files
            .iter()
            .filter_map(|x| match x {
                EContentFileType::Texture(texture) => {
                    if texture.borrow().is_virtual_texture {
                        Some(x.get_url())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    }

    fn process_ui_event(
        &mut self,
        window: &mut winit::window::Window,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let _span = tracy_client::span!();

        let egui_winit_state = &mut self.egui_winit_state;

        let click_event = self.editor_ui.build(
            egui_winit_state.egui_ctx(),
            &mut self.data_source,
            &mut self.model_loader,
        );

        self.process_top_menu_event(window, click_event.menu_event, event_loop_window_target);
        self.process_click_asset_event(click_event.click_aseet, event_loop_window_target);
        self.process_content_item_property_view_event();
        self.process_content_browser_event(
            click_event.content_browser_event,
            event_loop_window_target,
        );
        self.process_debug_texture_view_event(click_event.debug_textures_view_event);
        self.process_level_view_click_event(click_event.click_actor);
        self.process_project_settings_event(click_event.project_settings_event);
        self.process_object_property_view_event(click_event.object_property_view_event);
        self.process_gizmo_event(click_event.gizmo_event);
    }

    fn get_all_content_names(&self) -> Vec<String> {
        let content_data_source = &self.data_source.content_data_source;
        let Some(current_folder) = &content_data_source.current_folder else {
            return vec![];
        };
        let names = {
            let current_folder = current_folder.borrow();
            current_folder
                .files
                .iter()
                .map(|x| x.get_name())
                .collect::<Vec<String>>()
        };
        names
    }

    fn get_all_contents(&self) -> Vec<EContentFileType> {
        let content_data_source = &self.data_source.content_data_source;
        let Some(current_folder) = &content_data_source.current_folder else {
            return vec![];
        };
        let current_folder = current_folder.borrow();
        current_folder.files.to_vec()
    }

    pub fn copy_file_and_log<P: AsRef<Path> + Clone + Debug>(
        from: P,
        to: P,
    ) -> std::io::Result<u64> {
        let result = std::fs::copy(from.clone(), to.clone());
        match &result {
            Ok(_) => {
                log::trace!("Copy {:?} to {:?}", from, to);
            }
            Err(err) => {
                log::warn!("{}, can not copy {:?} to {:?}", err, from, to);
            }
        }
        result
    }

    fn save_current_project(&self) {
        if let Some(project_context) = self.project_context.as_ref() {
            let save_status = project_context.save();
            log::trace!("Save project: {:?}", save_status);
        }
    }

    fn process_top_menu_event(
        &mut self,
        window: &mut winit::window::Window,
        top_menu_event: Option<top_menu::EClickEventType>,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let Some(menu_event) = top_menu_event else {
            return;
        };

        match menu_event {
            top_menu::EClickEventType::NewProject(projevt_name) => {
                let _ = self
                    .event_loop_proxy
                    .send_event(ECustomEventType::OpenFileDialog(
                        EFileDialogType::NewProject(projevt_name.clone()),
                    ));
            }
            top_menu::EClickEventType::OpenProject => {
                let _ = self
                    .event_loop_proxy
                    .send_event(ECustomEventType::OpenFileDialog(
                        EFileDialogType::OpenProject,
                    ));
            }
            top_menu::EClickEventType::OpenRecentProject(file_path) => {
                let result = self.open_project(&file_path, window);
                log::trace!("Open project {result:?}");
            }
            top_menu::EClickEventType::SaveProject => {
                self.save_current_project();
            }
            top_menu::EClickEventType::Export => {
                if let Some(project_context) = self.project_context.as_mut() {
                    let result = project_context.export(&mut self.model_loader);
                    log::trace!("{:?}", result);
                }
            }
            top_menu::EClickEventType::OpenVisualStudioCode => {
                if let Some(project_context) = &self.project_context {
                    let path = project_context.get_project_folder_path();
                    let open_result = Self::open_project_workspace(&path);
                    log::trace!("{:?}", open_result);
                }
            }
            top_menu::EClickEventType::Build(build_config) => {
                let result = (|project_context: Option<&mut ProjectContext>| {
                    let project_context =
                        project_context.ok_or(anyhow!("project_context is null"))?;
                    let artifact_file_path = project_context.export(&mut self.model_loader)?;
                    let folder_path =
                        project_context.create_build_folder_if_not_exist(&build_config)?;
                    let target = rs_core_minimal::file_manager::get_engine_root_dir()
                        .join("rs_desktop_standalone/target");
                    let exe: PathBuf;
                    match build_config.build_type {
                        EBuildType::Debug => {
                            exe = target.join("debug/rs_desktop_standalone.exe");
                        }
                        EBuildType::Release => {
                            exe = target.join("release/rs_desktop_standalone.exe");
                        }
                    }
                    let to = folder_path.join("rs_desktop_standalone.exe");
                    Self::copy_file_and_log(exe, to)?;
                    let file_name = artifact_file_path
                        .file_name()
                        .ok_or(anyhow!("No file name"))?;
                    let to = folder_path.join(file_name);
                    Self::copy_file_and_log(artifact_file_path, to)?;
                    Ok::<(), anyhow::Error>(())
                })(self.project_context.as_mut());
                log::trace!("{:?}", result);
            }
            top_menu::EClickEventType::OpenWindow(window_type) => match window_type {
                top_menu::EWindowType::Asset => {
                    self.data_source.is_asset_folder_open = true;
                }
                top_menu::EWindowType::Content => {
                    self.data_source.content_data_source.is_open = true;
                }
                top_menu::EWindowType::Property => {
                    self.data_source.is_content_item_property_view_open = true;
                }
                top_menu::EWindowType::Level => {
                    self.data_source.is_level_view_open = true;
                }
                top_menu::EWindowType::ComsoleCmds => {
                    self.data_source.is_console_cmds_view_open = true;
                }
                top_menu::EWindowType::ObjectProperty => {
                    self.data_source.is_object_property_view_open = true;
                }
                top_menu::EWindowType::MultipleDrawUi => {
                    self.open_multiple_draw_ui_window(event_loop_window_target);
                }
                top_menu::EWindowType::DebugTexture => {
                    self.data_source.is_debug_texture_view_open = true;
                }
            },
            top_menu::EClickEventType::Tool(tool_type) => match tool_type {
                top_menu::EToolType::DebugShader => {
                    let result = Self::prepreocess_shader();
                    log::trace!("{:?}", result);
                }
            },
            top_menu::EClickEventType::OpenProjectSettings => {
                if self.project_context.is_some() {
                    self.data_source.project_settings_open = true;
                }
            }
            top_menu::EClickEventType::ViewMode(mode) => {
                self.engine.set_view_mode(mode);
            }
            top_menu::EClickEventType::Run => {
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let Ok(output_path) = project_context.export(&mut self.model_loader) else {
                    return;
                };
                let Some(output_path) = output_path
                    .canonicalize_slash()
                    .map(|x| x.to_str().map(|x| x.to_string()))
                    .ok()
                    .flatten()
                else {
                    return;
                };

                let debug_exe_path = file_manager::get_engine_root_dir()
                    .join("rs_desktop_standalone/target/debug/rs_desktop_standalone.exe");
                let release_exe_path = file_manager::get_engine_root_dir()
                    .join("rs_desktop_standalone/target/release/rs_desktop_standalone.exe");
                let executable_path = if release_exe_path.exists() {
                    Ok(release_exe_path)
                } else if debug_exe_path.exists() {
                    Ok(debug_exe_path)
                } else {
                    Err(crate::error::Error::IO(
                        std::io::ErrorKind::NotFound.into(),
                        None,
                    ))
                };
                let Ok(executable_path) = executable_path else {
                    return;
                };

                let mut command = std::process::Command::new(executable_path);
                command.arg("-i");
                command.arg(output_path);
                let output = command.output();
                log::trace!("{:?}", output);
            }
            top_menu::EClickEventType::DebugShading(ty) => {
                self.player_viewport.set_debug_shading(ty);
            }
            top_menu::EClickEventType::PlayStandalone => {
                let Some(_) = self.project_context.as_mut() else {
                    return;
                };
                self.open_standalone_window(
                    event_loop_window_target,
                    StandaloneSimulationType::Single,
                );
            }
            top_menu::EClickEventType::PlayAsServer => {
                let players = self.data_source.multiple_players;
                self.open_standalone_window(
                    event_loop_window_target,
                    StandaloneSimulationType::MultiplePlayer(MultiplePlayerOptions {
                        server_socket_addr: DEFAULT_SERVER_ADDR,
                        is_server: true,
                        players: players,
                    }),
                );
            }
        }
    }

    fn process_debug_texture_view_event(
        &mut self,
        event: Option<debug_textures_view::EClickEventType>,
    ) {
        let Some(event) = event else {
            return;
        };

        match event {
            debug_textures_view::EClickEventType::Selected(texture_url) => {
                let rm = ResourceManager::default();
                if rm.get_ui_texture_by_url(&texture_url).is_some() {
                    return;
                }

                let visualization_url =
                    build_built_in_resouce_url("ShadowDepthTextureVisualization").unwrap();

                if texture_url.scheme() == rs_engine::BUILT_IN_RESOURCE
                    && texture_url.host()
                        == Some(url::Host::Domain("PlayerViewport.ShadowDepthTexture"))
                    && rm.get_texture_by_url(&visualization_url).is_none()
                {
                    let texture_handle = self.engine.create_texture(
                        &visualization_url,
                        TextureDescriptorCreateInfo {
                            label: Some("ShadowDepthTextureVisualization".to_string()),
                            size: wgpu::Extent3d {
                                width: 1024,
                                height: 1024,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage: wgpu::TextureUsages::COPY_SRC
                                | wgpu::TextureUsages::STORAGE_BINDING
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                            view_formats: None,
                        },
                    );

                    self.engine.create_ui_texture(
                        rm.next_ui_texture(visualization_url.clone()),
                        texture_handle,
                    );

                    self.editor_ui.debug_textures_view.all_texture_urls = rm.get_texture_urls();
                }
            }
        }
    }

    fn process_project_settings_event(
        &mut self,
        event: Option<crate::ui::project_settings::EEventType>,
    ) {
        let Some(event) = event else {
            return;
        };
        match event {
            crate::ui::project_settings::EEventType::AntialiasType(ty) => {
                self.player_viewport
                    .on_antialias_type_changed(ty, &mut self.engine);
            }
        }
    }

    fn process_level_view_click_event(
        &mut self,
        level_view_event: Option<crate::ui::level_view::EClickEventType>,
    ) {
        let Some(event) = level_view_event else {
            return;
        };
        let Some(opened_level) = self.data_source.level.as_mut() else {
            return;
        };
        match event {
            crate::ui::level_view::EClickEventType::SingleClickActor(actor) => {
                self.editor_ui.object_property_view.selected_object =
                    Some(ESelectedObjectType::Actor(actor));
                self.data_source.is_object_property_view_open = true;
            }
            crate::ui::level_view::EClickEventType::SingleClickSceneNode(scene_node) => {
                self.editor_ui.object_property_view.selected_object =
                    Some(ESelectedObjectType::SceneNode(scene_node));
                self.data_source.is_object_property_view_open = true;
            }
            crate::ui::level_view::EClickEventType::CreateDirectionalLight => {
                let size = 5.0;
                let far = 50.0;
                let mut opened_level = opened_level.borrow_mut();
                let names = opened_level
                    .directional_lights
                    .iter()
                    .map(|x| x.borrow().name.clone())
                    .collect();
                let mut light = DirectionalLight::new(
                    make_unique_name(names, "DirectionalLight"),
                    -size,
                    size,
                    -size,
                    size,
                    0.01,
                    far,
                );
                light.initialize(&mut self.engine, &mut self.player_viewport);
                opened_level
                    .directional_lights
                    .push(SingleThreadMut::new(light));
            }
            crate::ui::level_view::EClickEventType::DirectionalLight(light) => {
                self.editor_ui.object_property_view.selected_object =
                    Some(ESelectedObjectType::DirectionalLight(light));
                self.data_source.is_object_property_view_open = true;
            }
            crate::ui::level_view::EClickEventType::CreateCameraComponent(parent_node) => {
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let content = project_context.project.content.clone();
                let content = content.borrow_mut();
                let mut camera_component =
                    CameraComponent::new("Camera".to_string(), glam::Mat4::IDENTITY);
                camera_component.initialize(
                    &mut self.engine,
                    &content.files,
                    &mut self.player_viewport,
                );
                let camera_component = SingleThreadMut::new(camera_component);
                let mut parent_node = parent_node.borrow_mut();
                parent_node.childs.push(SingleThreadMut::new(SceneNode {
                    component: rs_engine::scene_node::EComponentType::CameraComponent(
                        camera_component,
                    ),
                    childs: vec![],
                }));
            }
            crate::ui::level_view::EClickEventType::DeleteNode(actor, node) => {
                actor.borrow_mut().remove_node(node);
            }
            crate::ui::level_view::EClickEventType::DeleteDirectionalLight(light) => {
                opened_level.borrow_mut().delete_light(light);
            }
            crate::ui::level_view::EClickEventType::DeleteActor(actor) => {
                opened_level.borrow_mut().delete_actor(actor);
            }
            crate::ui::level_view::EClickEventType::CreateSceneComponent(parent_node) => {
                let mut parent_node = parent_node.borrow_mut();
                let names = parent_node
                    .childs
                    .iter()
                    .map(|x| x.borrow().get_name())
                    .collect();
                let new_name = make_unique_name(names, "Scene");
                parent_node.childs.push(SceneNode::new_sp(new_name));
            }
            crate::ui::level_view::EClickEventType::CopyPath(actor, node) => {
                if let Some(node_path) = actor.borrow_mut().find_path_by_node(node) {
                    self.editor_ui.egui_context.copy_text(node_path);
                }
            }
            crate::ui::level_view::EClickEventType::CreateActor => {
                let Some(active_level) = self.data_source.level.as_mut() else {
                    return;
                };
                let mut active_level = active_level.borrow_mut();
                let _ = active_level.create_and_insert_actor();
            }
            crate::ui::level_view::EClickEventType::CreateCameraHere => {
                let Some(active_level) = self.data_source.level.as_mut() else {
                    return;
                };
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let content = project_context.project.content.clone();
                let content = content.borrow_mut();
                let mut active_level = active_level.borrow_mut();
                let new_actor = active_level.create_and_insert_actor();
                let new_actor = new_actor.borrow_mut();
                let mut scene_node = new_actor.scene_node.borrow_mut();
                scene_node
                    .set_transformation(self.player_viewport.camera.get_world_transformation());
                let mut camera_component =
                    CameraComponent::new("Camera".to_string(), glam::Mat4::IDENTITY);
                camera_component.initialize(
                    &mut self.engine,
                    &content.files,
                    &mut self.player_viewport,
                );
                let camera_component = SingleThreadMut::new(camera_component);
                scene_node.childs.push(SingleThreadMut::new(SceneNode {
                    component: rs_engine::scene_node::EComponentType::CameraComponent(
                        camera_component,
                    ),
                    childs: vec![],
                }));
            }
            crate::ui::level_view::EClickEventType::CreateCollisionComponent(_, parent_node) => {
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let content = project_context.project.content.clone();
                let content = content.borrow_mut();
                let mut parent_node = parent_node.borrow_mut();
                let names = parent_node
                    .childs
                    .iter()
                    .map(|x| x.borrow().get_name())
                    .collect();
                let new_name = make_unique_name(names, "Collision");
                let collision_component =
                    CollisionComponent::new_scene_node(new_name, glam::Mat4::IDENTITY);
                {
                    let mut collision_component = collision_component.borrow_mut();
                    collision_component.initialize(
                        &mut self.engine,
                        &content.files,
                        &mut self.player_viewport,
                    );
                }
                parent_node.childs.push(collision_component);
            }
            crate::ui::level_view::EClickEventType::DuplicateActor(actor) => {
                let Some(active_level) = self.data_source.level.as_mut() else {
                    return;
                };
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };

                let mut active_level = active_level.borrow_mut();
                let content = project_context.project.content.clone();
                let content = content.borrow_mut();
                active_level.duplicate_actor(
                    actor,
                    &mut self.engine,
                    &content.files,
                    &mut self.player_viewport,
                );
            }
            crate::ui::level_view::EClickEventType::CreateSpotLightComponent(parent_node) => {
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let content = project_context.project.content.clone();
                let content = content.borrow_mut();
                let mut parent_node = parent_node.borrow_mut();
                let names = parent_node
                    .childs
                    .iter()
                    .map(|x| x.borrow().get_name())
                    .collect();
                let new_name = make_unique_name(names, "SpotLight");
                let spot_light_component =
                    SpotLightComponent::new_scene_node(new_name, glam::Mat4::IDENTITY);
                {
                    let mut spot_light_component = spot_light_component.borrow_mut();
                    spot_light_component.initialize(
                        &mut self.engine,
                        &content.files,
                        &mut self.player_viewport,
                    );
                }
                parent_node.childs.push(spot_light_component);
            }
            crate::ui::level_view::EClickEventType::CreatePointLightComponent(parent_node) => {
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let content = project_context.project.content.clone();
                let content = content.borrow_mut();
                let mut parent_node = parent_node.borrow_mut();
                let names = parent_node
                    .childs
                    .iter()
                    .map(|x| x.borrow().get_name())
                    .collect();
                let new_name = make_unique_name(names, "PointLight");
                let point_light_component =
                    PointLightComponent::new_scene_node(new_name, glam::Mat4::IDENTITY);
                {
                    let mut point_light_component = point_light_component.borrow_mut();
                    point_light_component.initialize(
                        &mut self.engine,
                        &content.files,
                        &mut self.player_viewport,
                    );
                }
                parent_node.childs.push(point_light_component);
            }
        }
    }

    fn process_content_browser_event(
        &mut self,
        content_browser_event: Option<content_browser::EClickEventType>,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let Some(event) = content_browser_event else {
            return;
        };
        let Some(current_folder) = &self.data_source.content_data_source.current_folder else {
            return;
        };
        match event {
            content_browser::EClickEventType::CreateFolder => {
                let new_folder_name = &self.data_source.content_data_source.new_folder_name;
                let names: Vec<String> = current_folder
                    .borrow()
                    .folders
                    .iter()
                    .map(|x| x.borrow().name.clone())
                    .collect();
                if names.contains(new_folder_name) {
                    return;
                }
                let new_folder = ContentFolder::new(new_folder_name, Some(current_folder.clone()));
                current_folder
                    .borrow_mut()
                    .folders
                    .push(Rc::new(RefCell::new(new_folder)));
            }
            content_browser::EClickEventType::Back => {
                let parent_folder = current_folder.borrow().parent_folder.clone();
                let Some(parent_folder) = parent_folder else {
                    return;
                };
                self.data_source.content_data_source.current_folder = Some(parent_folder.clone());
            }
            content_browser::EClickEventType::OpenFolder(folder) => {
                self.data_source.content_data_source.current_folder = Some(folder.clone());
            }
            content_browser::EClickEventType::OpenFile(file) => {
                self.editor_ui.content_item_property_view.content = Some(file.clone());
                self.data_source.is_content_item_property_view_open = true;
                match file {
                    EContentFileType::StaticMesh(static_mesh) => {
                        self.open_static_mesh_window(
                            &mut static_mesh.borrow_mut(),
                            event_loop_window_target,
                        );
                    }
                    EContentFileType::SkeletonMesh(skeleton_mesh) => {
                        self.open_skin_mesh_window(
                            &mut skeleton_mesh.borrow_mut(),
                            event_loop_window_target,
                        );
                    }
                    EContentFileType::SkeletonAnimation(_) => {}
                    EContentFileType::Skeleton(_) => {}
                    EContentFileType::Texture(_) => {}
                    EContentFileType::Level(_) => {}
                    EContentFileType::Material(material) => {
                        self.open_material_window(event_loop_window_target, Some(material.clone()));
                    }
                    EContentFileType::IBL(_) => {}
                    EContentFileType::ParticleSystem(particle_system) => {
                        self.open_particle_window(event_loop_window_target, particle_system);
                    }
                    EContentFileType::Sound(_) => todo!(),
                    EContentFileType::Curve(curve) => {
                        self.data_source.opened_curve = Some(curve);
                        self.data_source.is_content_item_property_view_open = false;
                    }
                    EContentFileType::BlendAnimations(blend_animation) => {
                        self.open_blend_animation_ui_window(
                            blend_animation,
                            event_loop_window_target,
                        );
                    }
                    EContentFileType::MaterialParamentersCollection(_) => {}
                }
            }
            content_browser::EClickEventType::SingleClickFile(file) => {
                self.data_source.content_data_source.highlight_file = Some(file.clone());
            }
            content_browser::EClickEventType::CreateMaterial => {
                let names = self.get_all_content_names();
                let name = make_unique_name(
                    names,
                    &self.data_source.content_data_source.new_material_name,
                );
                let Some(project_context) = &mut self.project_context else {
                    return;
                };

                let mut material = rs_engine::content::material::Material::new(
                    build_content_file_url(&name).unwrap(),
                    build_asset_url(format!("material/{}", &name)).unwrap(),
                );
                let resolve_result = material_view::MaterialView::default_resolve().unwrap();
                {
                    let mut shader_code = HashMap::new();
                    let mut material_info = HashMap::new();
                    for (k, v) in resolve_result.iter() {
                        shader_code.insert(k.clone(), v.shader_code.clone());
                        material_info.insert(k.clone(), v.material_info.clone());
                    }
                    let handle = self.engine.create_material(shader_code);
                    material.set_pipeline_handle(handle);
                    material.set_material_info(material_info);
                }
                let material_editor = crate::material::Material::new(material.asset_url.clone(), {
                    let mut snarl = egui_snarl::Snarl::new();
                    let node = MaterialNode {
                        node_type: EMaterialNodeType::Sink(Default::default()),
                    };
                    snarl.insert_node(egui::pos2(0.0, 0.0), node);
                    snarl
                });
                if self
                    .engine
                    .get_settings()
                    .render_setting
                    .is_enable_dump_material_shader_code
                {
                    if let Err(err) = Self::write_debug_shader(&material_editor, &resolve_result) {
                        log::warn!("{}", err);
                    }
                }
                project_context
                    .project
                    .materials
                    .push(Rc::new(RefCell::new(material_editor)));
                project_context
                    .project
                    .content
                    .borrow_mut()
                    .files
                    .push(EContentFileType::Material(Rc::new(RefCell::new(material))));

                let mut materials = self.editor_ui.object_property_view.materials.borrow_mut();
                if !materials.contains(&build_content_file_url(&name).unwrap()) {
                    materials.push(build_content_file_url(&name).unwrap());
                }
            }
            content_browser::EClickEventType::CreateIBL => {
                let names = self.get_all_content_names();
                let name =
                    make_unique_name(names, &self.data_source.content_data_source.new_ibl_name);

                let new_ibl =
                    rs_engine::content::ibl::IBL::new(build_content_file_url(&name).unwrap());
                let new_ibl = SingleThreadMut::new(new_ibl);
                let Some(project_context) = &mut self.project_context else {
                    return;
                };
                project_context
                    .project
                    .content
                    .borrow_mut()
                    .files
                    .push(EContentFileType::IBL(new_ibl));
            }
            content_browser::EClickEventType::CreateParticleSystem => {
                let names = self.get_all_content_names();
                let name = make_unique_name(
                    names,
                    &self.data_source.content_data_source.new_content_name,
                );
                let Some(project_context) = &mut self.project_context else {
                    return;
                };
                let particle_system = rs_engine::content::particle_system::ParticleSystem::new(
                    build_content_file_url(&name).unwrap(),
                );
                let particle_system = SingleThreadMut::new(particle_system);
                project_context
                    .project
                    .content
                    .borrow_mut()
                    .files
                    .push(EContentFileType::ParticleSystem(particle_system));
            }
            content_browser::EClickEventType::DeleteFile(content_file) => {
                let Some(project_context) = &mut self.project_context else {
                    return;
                };
                let content = project_context.project.content.clone();
                let mut content = content.borrow_mut();
                content
                    .files
                    .retain(|x| x.get_url() != content_file.get_url());
            }
            content_browser::EClickEventType::CreateCurve => {
                let names = self.get_all_content_names();
                let name = make_unique_name(
                    names,
                    &self.data_source.content_data_source.new_content_name,
                );
                let Some(project_context) = &mut self.project_context else {
                    return;
                };
                let curve =
                    rs_engine::content::curve::Curve::new(build_content_file_url(&name).unwrap());
                let curve = SingleThreadMut::new(curve);
                project_context
                    .project
                    .content
                    .borrow_mut()
                    .files
                    .push(EContentFileType::Curve(curve));
            }
            content_browser::EClickEventType::Rename(mut content_file_type, new_name) => {
                let names = self.get_all_content_names();
                if names.contains(&new_name) {
                    return;
                }
                content_file_type.set_name(new_name);
            }
            content_browser::EClickEventType::CreateBlendAnimations => {
                let names = self.get_all_content_names();
                let name = make_unique_name(
                    names,
                    &self.data_source.content_data_source.new_content_name,
                );
                let Some(project_context) = &mut self.project_context else {
                    return;
                };
                let Ok(content_url) = build_content_file_url(&name) else {
                    return;
                };
                let blend_animation =
                    rs_engine::content::blend_animations::BlendAnimations::new(content_url);
                let blend_animation = SingleThreadMut::new(blend_animation);
                project_context
                    .project
                    .content
                    .borrow_mut()
                    .files
                    .push(EContentFileType::BlendAnimations(blend_animation));
            }
            content_browser::EClickEventType::CreateMaterialParametersCollection => {
                let names = self.get_all_content_names();
                let name = make_unique_name(
                    names,
                    &self.data_source.content_data_source.new_content_name,
                );
                let Some(project_context) = &mut self.project_context else {
                    return;
                };
                let Ok(content_url) = build_content_file_url(&name) else {
                    return;
                };
                let material_paramenters_collection =
                    rs_engine::content::material_paramenters_collection::MaterialParamentersCollection::new(content_url);
                let material_paramenters_collection =
                    SingleThreadMut::new(material_paramenters_collection);
                project_context.project.content.borrow_mut().files.push(
                    EContentFileType::MaterialParamentersCollection(
                        material_paramenters_collection,
                    ),
                );
            }
            content_browser::EClickEventType::Detail(file) => {
                self.editor_ui.content_item_property_view.content = Some(file.clone());
                self.data_source.is_content_item_property_view_open = true;
            }
        }
    }

    fn process_click_asset_event(
        &mut self,
        click_aseet: Option<asset_view::EClickItemType>,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let Some(click_aseet) = click_aseet else {
            return;
        };
        match click_aseet {
            asset_view::EClickItemType::Folder(folder) => {
                self.data_source.current_asset_folder = Some(folder);
            }
            asset_view::EClickItemType::File(asset_file) => {
                self.data_source.highlight_asset_file = Some(asset_file.clone());
                if asset_file.get_file_type() == EFileType::Mp4 {
                    self.open_media_window(asset_file.path, event_loop_window_target);
                } else if asset_file.get_file_type().is_model() {
                    let _ = self
                        .model_loader
                        .load_scene_from_file_and_cache(&asset_file.path);
                    self.data_source.model_scene_view_data = Default::default();
                    self.data_source.model_scene_view_data.model_scene = Some(asset_file.path);
                }
            }
            asset_view::EClickItemType::Back => todo!(),
            asset_view::EClickItemType::SingleClickFile(asset_file) => {
                self.data_source.highlight_asset_file = Some(asset_file)
            }
            asset_view::EClickItemType::CreateTexture(asset_file) => {
                if let Some(project_context) = self.project_context.as_mut() {
                    let asset_folder_path = project_context.get_asset_folder_path();
                    let image_reference: PathBuf = {
                        if asset_file.path.starts_with(asset_folder_path.clone()) {
                            asset_file
                                .path
                                .strip_prefix(asset_folder_path)
                                .unwrap()
                                .to_path_buf()
                        } else {
                            asset_file.path
                        }
                    };
                    if let Some(current_folder) =
                        &self.data_source.content_data_source.current_folder
                    {
                        let mut current_folder = current_folder.borrow_mut();
                        let folder_url = current_folder.get_url();
                        let url = folder_url.join(&asset_file.name).unwrap();
                        let mut texture_file = TextureFile::new(url);
                        // texture_file.image_reference = Some(image_reference);
                        texture_file.set_image_reference_path(image_reference);
                        log::trace!("Create texture: {:?}", &texture_file.url.as_str());
                        let texture_file =
                            EContentFileType::Texture(Rc::new(RefCell::new(texture_file)));
                        Self::content_load_resources(
                            &mut self.engine,
                            &mut self.model_loader,
                            project_context,
                            vec![texture_file.clone()],
                        );
                        current_folder.files.push(texture_file.clone());
                    }
                }
            }
            asset_view::EClickItemType::CreateMediaSource(_) => todo!(),
            asset_view::EClickItemType::PlaySound(_) => {
                //
                todo!()
            }
            asset_view::EClickItemType::CreateSound(asset_file) => {
                let names = self.get_all_content_names();
                let Some(project_context) = self.project_context.as_mut() else {
                    return;
                };
                let Some(current_folder) = &self.data_source.content_data_source.current_folder
                else {
                    return;
                };
                let asset_folder_path = project_context.get_asset_folder_path();

                let relative_path: PathBuf = {
                    if asset_file.path.starts_with(asset_folder_path.clone()) {
                        asset_file
                            .path
                            .strip_prefix(asset_folder_path)
                            .unwrap()
                            .to_path_buf()
                    } else {
                        asset_file.path
                    }
                };
                let new_name = make_unique_name(names, &asset_file.name);

                let mut current_folder = current_folder.borrow_mut();
                let folder_url = current_folder.get_url();
                let url = folder_url.join(&new_name).unwrap();

                let sound = rs_engine::content::sound::Sound::new(url, relative_path);
                let content = EContentFileType::Sound(Rc::new(RefCell::new(sound)));
                Self::content_load_resources(
                    &mut self.engine,
                    &mut self.model_loader,
                    project_context,
                    vec![content.clone()],
                );
                current_folder.files.push(content);
            }
            asset_view::EClickItemType::ImportAsActor(asset_file) => {
                let result = self.open_model_file(asset_file.path.clone());
                log::trace!("{:?}", result);
            }
        }
    }

    fn process_content_item_property_view_event(&mut self) {
        let Some(event) = &self.editor_ui.content_item_property_view.click else {
            return;
        };
        match event {
            content_item_property_view::EEventType::IBL(ibl, old, new) => {
                let url = ibl.borrow().url.clone();
                let Some(new) = new.as_ref() else {
                    return;
                };
                let result = (|| {
                    let project_context = self.project_context.as_ref().ok_or(anyhow!(""))?;
                    log::trace!("{:?}", new);
                    let file_path = project_context.get_project_folder_path().join(new);
                    if !file_path.exists() {
                        return Err(anyhow!("The file is not exist"));
                    }
                    let is_contains = self
                        .engine
                        .get_resource_manager()
                        .get_ibl_textures()
                        .contains_key(&url);
                    if !is_contains {
                        Self::load_ibl_content_resource(
                            &mut self.engine,
                            project_context,
                            ibl.clone(),
                        )?;
                    }
                    Ok(())
                })();
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        log::warn!("{}", err);
                        ibl.borrow_mut().image_reference = old.clone();
                    }
                }
            }
            content_item_property_view::EEventType::IsVirtualTexture(
                texture_file,
                is_virtual_texture,
            ) => {
                let result: anyhow::Result<()> = (|| {
                    if !is_virtual_texture {
                        return Ok(());
                    }
                    let project_context = self
                        .project_context
                        .as_ref()
                        .ok_or(anyhow!("No project context"))?;
                    let virtual_texture_cache_dir =
                        project_context.try_create_virtual_texture_cache_dir()?;
                    let project_folder_path = &project_context.get_project_folder_path();

                    let virtual_cache_name = texture_file
                        .borrow()
                        .get_pref_virtual_cache_name(project_folder_path)?;
                    texture_file.borrow_mut().create_virtual_texture_cache(
                        project_folder_path,
                        &virtual_texture_cache_dir.join(virtual_cache_name.clone()),
                        Some(rs_artifact::EEndianType::Little),
                        256,
                    )?;
                    log::trace!("virtual_cache_name: {}", virtual_cache_name);
                    texture_file.borrow_mut().virtual_image_reference = Some(virtual_cache_name);
                    Ok(())
                })();
                log::trace!("{:?}", result);
            }
            content_item_property_view::EEventType::SDF2D(texture) => {
                let result: anyhow::Result<()> = (|| {
                    let texture = texture.borrow();
                    let image_reference = texture.image_reference.as_ref().ok_or(anyhow!(""))?;
                    let project_context = self.project_context.as_ref().ok_or(anyhow!(""))?;
                    let path = project_context.get_asset_path_by_url(image_reference);
                    let image = image::open(path)?;
                    let image = image.to_rgba8();
                    self.engine.sdf2d(image);
                    Ok(())
                })();
                log::trace!("{:?}", result);
            }
            content_item_property_view::EEventType::UpdateMaterialParamentersCollection(
                update_info,
            ) => {
                let mut material_paramenters_collection = update_info.0.borrow_mut();
                material_paramenters_collection.fields = update_info.1.fields.clone();
                material_paramenters_collection.initialize(&mut self.engine);
            }
            content_item_property_view::EEventType::UpdateStaticMeshEnableMultiresolution(
                static_mesh,
                _,
                new_value,
            ) => {
                let mut static_mesh = static_mesh.borrow_mut();
                static_mesh.is_enable_multiresolution = *new_value;
                let project_context = self.project_context.as_ref().unwrap();

                if let Err(err) =
                    Self::create_multi_res_mesh_cache_non_blocking(project_context, &static_mesh)
                {
                    log::warn!("{}", err);
                }
            }
        }
    }

    fn process_object_property_view_event(
        &mut self,
        event: Option<object_property_view::EEventType>,
    ) {
        let Some(event) = event else {
            return;
        };
        let Some(active_level) = self.data_source.level.as_mut() else {
            return;
        };
        match event {
            object_property_view::EEventType::UpdateMaterial(update_material) => {
                match update_material.selected_object {
                    ESelectedObjectType::Actor(_) => unimplemented!(),
                    ESelectedObjectType::DirectionalLight(_) => unimplemented!(),
                    ESelectedObjectType::SceneNode(scene_node) => {
                        let scene_node = scene_node.borrow_mut();
                        match &scene_node.component {
                            rs_engine::scene_node::EComponentType::SceneComponent(_) => {
                                unimplemented!()
                            }
                            rs_engine::scene_node::EComponentType::StaticMeshComponent(
                                static_mesh_component,
                            ) => {
                                let files = if let Some(folder) =
                                    &self.data_source.content_data_source.current_folder
                                {
                                    folder.borrow().files.clone()
                                } else {
                                    vec![]
                                };
                                let mut static_mesh_component = static_mesh_component.borrow_mut();
                                static_mesh_component.set_material(
                                    &mut self.engine,
                                    update_material.new,
                                    &files,
                                    &mut self.player_viewport,
                                );
                            }
                            rs_engine::scene_node::EComponentType::SkeletonMeshComponent(
                                skeleton_mesh_component,
                            ) => {
                                let files = if let Some(folder) =
                                    &self.data_source.content_data_source.current_folder
                                {
                                    folder.borrow().files.clone()
                                } else {
                                    vec![]
                                };
                                let mut skeleton_mesh_component =
                                    skeleton_mesh_component.borrow_mut();
                                if let Some(url) = update_material.new {
                                    skeleton_mesh_component.set_material(
                                        &mut self.engine,
                                        url,
                                        &files,
                                        &mut self.player_viewport,
                                    );
                                }
                            }
                            rs_engine::scene_node::EComponentType::CameraComponent(_) => {
                                unimplemented!()
                            }
                            rs_engine::scene_node::EComponentType::CollisionComponent(_) => {
                                unimplemented!()
                            }
                            rs_engine::scene_node::EComponentType::SpotLightComponent(_) => {
                                unimplemented!()
                            }
                            rs_engine::scene_node::EComponentType::PointLightComponent(_) => {
                                unimplemented!()
                            }
                        }
                    }
                }
            }
            object_property_view::EEventType::UpdateDirectionalLight(
                directional_light,
                left,
                right,
                top,
                bottom,
                far,
            ) => {
                let mut directional_light = directional_light.borrow_mut();
                directional_light.left = left;
                directional_light.right = right;
                directional_light.top = top;
                directional_light.bottom = bottom;
                directional_light.far = far;
                directional_light.remake_preview(&mut self.engine, &mut self.player_viewport);
            }
            object_property_view::EEventType::UpdateAnimation(update_animation) => {
                match update_animation.selected_object {
                    ESelectedObjectType::Actor(_) => unimplemented!(),
                    ESelectedObjectType::DirectionalLight(_) => unimplemented!(),
                    ESelectedObjectType::SceneNode(scene_node) => {
                        let scene_node = scene_node.borrow_mut();
                        match &scene_node.component {
                            rs_engine::scene_node::EComponentType::SkeletonMeshComponent(
                                skeleton_mesh_component,
                            ) => {
                                let mut skeleton_mesh_component =
                                    skeleton_mesh_component.borrow_mut();
                                let files = if let Some(folder) =
                                    &self.data_source.content_data_source.current_folder
                                {
                                    folder.borrow().files.clone()
                                } else {
                                    vec![]
                                };
                                skeleton_mesh_component.set_animation(
                                    update_animation.new,
                                    self.engine.get_resource_manager().clone(),
                                    &files,
                                );
                            }
                            _ => {
                                unimplemented!()
                            }
                        }
                    }
                }
            }
            object_property_view::EEventType::ChangeName(selected_object_type, new_name) => {
                // let opened_level = opened_level.borrow();
                if !rs_core_minimal::misc::is_valid_name(&new_name) {
                    return;
                }
                match selected_object_type {
                    ESelectedObjectType::Actor(actor) => {
                        actor.borrow_mut().name = new_name;
                    }
                    ESelectedObjectType::SceneNode(scene_node) => {
                        scene_node.borrow_mut().set_name(new_name);
                    }
                    ESelectedObjectType::DirectionalLight(componenet) => {
                        componenet.borrow_mut().name = new_name;
                    }
                }
            }
            object_property_view::EEventType::UpdateStaticMesh(update_static_mesh) => {
                match update_static_mesh.selected_object {
                    ESelectedObjectType::SceneNode(scene_node) => {
                        let mut scene_node = scene_node.borrow_mut();
                        match &mut scene_node.component {
                            rs_engine::scene_node::EComponentType::StaticMeshComponent(
                                static_mesh_component,
                            ) => {
                                let mut static_mesh_component = static_mesh_component.borrow_mut();
                                let static_mesh_url = update_static_mesh.new;
                                let files = if let Some(folder) =
                                    &self.data_source.content_data_source.current_folder
                                {
                                    folder.borrow().files.clone()
                                } else {
                                    vec![]
                                };
                                static_mesh_component.set_static_mesh_url(
                                    static_mesh_url,
                                    self.engine.get_resource_manager().clone(),
                                    &mut self.engine,
                                    &files,
                                    &mut self.player_viewport,
                                );
                                let mut active_level = active_level.borrow_mut();
                                let physics = active_level.get_physics_mut();
                                if let Some(physics) = physics {
                                    static_mesh_component.initialize_physics(
                                        &mut physics.rigid_body_set,
                                        &mut physics.collider_set,
                                    );
                                }
                            }
                            _ => unimplemented!(),
                        }
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
            object_property_view::EEventType::UpdateIsEnableMultiresolution(
                selected_object_type,
                old,
                new,
            ) => {
                let _ = old;
                match selected_object_type {
                    ESelectedObjectType::SceneNode(scene_node) => {
                        let mut scene_node = scene_node.borrow_mut();
                        match &mut scene_node.component {
                            rs_engine::scene_node::EComponentType::StaticMeshComponent(
                                static_mesh_component,
                            ) => {
                                let mut static_mesh_component = static_mesh_component.borrow_mut();
                                static_mesh_component.is_enable_multiresolution = new;
                            }
                            _ => unimplemented!(),
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }

    fn process_gizmo_event(&mut self, event: Option<GizmoEvent>) {
        let Some(event) = event else {
            return;
        };
        let Some(active_level) = self.data_source.level.clone() else {
            return;
        };
        let mut active_level = active_level.borrow_mut();

        let gizmo_final_transformation: Option<glam::Mat4> =
            event.gizmo_result.map(|(_, transforms)| {
                let transform = transforms[0];
                let gizmo_final_transformation = glam::DMat4::from_scale_rotation_translation(
                    transform.scale.into(),
                    transform.rotation.into(),
                    transform.translation.into(),
                )
                .as_mat4();
                gizmo_final_transformation
            });

        match event.selected_object {
            ESelectedObjectType::Actor(_) => {}
            ESelectedObjectType::SceneNode(secne_node) => {
                let mut secne_node = secne_node.borrow_mut();
                let component = &mut secne_node.component;
                match component {
                    rs_engine::scene_node::EComponentType::SceneComponent(component) => {
                        let mut component = component.borrow_mut();
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let parent_final_transformation =
                                component.get_parent_final_transformation();
                            let model_matrix = component.get_transformation_mut();
                            *model_matrix =
                                parent_final_transformation.inverse() * gizmo_final_transformation;
                        }
                    }
                    rs_engine::scene_node::EComponentType::StaticMeshComponent(component) => {
                        let mut component = component.borrow_mut();
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let parent_final_transformation =
                                component.get_parent_final_transformation();
                            let model_matrix = component.get_transformation_mut();
                            *model_matrix =
                                parent_final_transformation.inverse() * gizmo_final_transformation;
                            component.set_apply_simulate(false);
                        } else {
                            component.set_apply_simulate(true);
                        }
                    }
                    rs_engine::scene_node::EComponentType::SkeletonMeshComponent(component) => {
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let mut component = component.borrow_mut();
                            *component.get_transformation_mut() = gizmo_final_transformation;
                        }
                    }
                    rs_engine::scene_node::EComponentType::CameraComponent(component) => {
                        let mut component = component.borrow_mut();
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let parent_final_transformation =
                                component.get_parent_final_transformation();
                            let model_matrix = component.get_transformation_mut();
                            *model_matrix =
                                parent_final_transformation.inverse() * gizmo_final_transformation;
                        }
                    }
                    rs_engine::scene_node::EComponentType::CollisionComponent(component) => {
                        let mut component = component.borrow_mut();
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let parent_final_transformation =
                                component.get_parent_final_transformation();
                            let model_matrix = component.get_transformation_mut();
                            *model_matrix =
                                parent_final_transformation.inverse() * gizmo_final_transformation;
                        }
                    }
                    rs_engine::scene_node::EComponentType::SpotLightComponent(component) => {
                        let mut component = component.borrow_mut();
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let parent_final_transformation =
                                component.get_parent_final_transformation();
                            let model_matrix =
                                parent_final_transformation.inverse() * gizmo_final_transformation;
                            component.set_transformation(model_matrix);
                        }
                    }
                    rs_engine::scene_node::EComponentType::PointLightComponent(component) => {
                        let mut component = component.borrow_mut();
                        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                            let parent_final_transformation =
                                component.get_parent_final_transformation();
                            let model_matrix =
                                parent_final_transformation.inverse() * gizmo_final_transformation;
                            component.set_transformation(model_matrix);
                        }
                    }
                }
                let level_physics = active_level.get_physics_mut();
                secne_node.notify_transformation_updated(level_physics);
            }

            ESelectedObjectType::DirectionalLight(component) => {
                if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                    let mut component = component.borrow_mut();
                    *component.get_transformation_mut() = gizmo_final_transformation;
                }
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
