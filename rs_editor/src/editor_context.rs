use crate::{
    build_config::EBuildType,
    content_folder::ContentFolder,
    custom_event::{ECustomEventType, EFileDialogType},
    data_source::{AssetFile, AssetFolder, DataSource},
    editor_ui::{EditorUI, GizmoEvent},
    material_resolve,
    model_loader::ModelLoader,
    project::Project,
    project_context::{EFolderUpdateType, ProjectContext},
    ui::{
        asset_view, content_browser, content_item_property_view, debug_textures_view,
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
    },
    watch_shader::WatchShader,
    windows_manager::WindowsManager,
};
use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use rs_artifact::{material::MaterialInfo, sound::ESoundFileType};
use rs_core_minimal::{
    file_manager, name_generator::make_unique_name, path_ext::CanonicalizeSlashExt,
};
#[cfg(any(feature = "plugin_shared_crate"))]
use rs_engine::plugin::plugin_crate::Plugin;
use rs_engine::{
    build_asset_url, build_built_in_resouce_url, build_content_file_url,
    content::{content_file_type::EContentFileType, texture::TextureFile},
    directional_light::DirectionalLight,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
    player_viewport::PlayerViewport,
};
use rs_engine::{
    file_type::EFileType,
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
    static_virtual_texture_source::StaticVirtualTextureSource,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::{
    command::{RenderCommand, ScaleChangedInfo, TextureDescriptorCreateInfo},
    get_buildin_shader_dir,
};
use rs_render_types::MaterialOptions;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
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
}

struct MouseState {
    is_focus: bool,
    position: glam::Vec2,
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
    v8_runtime: Option<rs_v8_host::v8_runtime::V8Runtime>,
    frame_sync: FrameSync,
    model_loader: ModelLoader,
    window_manager: Rc<RefCell<WindowsManager>>,
    material_ui_window: Option<MaterialUIWindow>,
    particle_system_ui_window: Option<ParticleSystemUIWindow>,
    mesh_ui_window: Option<MeshUIWindow>,
    media_ui_window: Option<MediaUIWindow>,
    multiple_draw_ui_window: Option<MultipleDrawUiWindow>,
    standalone_ui_window: Option<StandaloneUiWindow>,
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

    pub fn new(
        window_id: isize,
        window: &winit::window::Window,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
        window_manager: Rc<RefCell<WindowsManager>>,
    ) -> anyhow::Result<EditorContext> {
        let _span = tracy_client::span!();
        rs_foundation::change_working_directory();
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
            is_flush_before_drop: false,
        });
        log::trace!(
            "Engine Root Dir: {:?}",
            rs_core_minimal::file_manager::get_engine_root_dir().canonicalize_slash()?
        );
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
            ProjectContext::pre_process_shaders(),
        )?;

        Self::insert_cmds(&mut engine);

        let mut data_source = DataSource::new();
        data_source.console_cmds = Some(engine.get_console_cmds());
        let editor_ui = EditorUI::new(egui_winit_state.egui_ctx());

        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let watch_shader = WatchShader::new(get_buildin_shader_dir())?;

        #[cfg(feature = "plugin_dotnet")]
        let donet_host = rs_dotnet_host::dotnet_runtime::DotnetRuntime::default().ok();
        #[cfg(feature = "plugin_v8")]
        let v8_runtime = {
            let mut v8_runtime = rs_v8_host::v8_runtime::V8Runtime::new();
            let _ = v8_runtime.register_func_global();
            // let _ = v8_runtime.register_engine_global(&mut editor_context.engine);
            v8_runtime
        };

        let virtual_texture_source_infos = engine.get_virtual_texture_source_infos();
        let player_viewport = PlayerViewport::new(
            window_id,
            window_width,
            window_height,
            ResourceManager::default()
                .get_builtin_resources()
                .global_sampler_handle
                .clone(),
            &mut engine,
            virtual_texture_source_infos,
            EInputMode::UI,
        );

        let editor_context = EditorContext {
            event_loop_proxy,
            engine,
            egui_winit_state,
            data_source,
            project_context: None,
            virtual_key_code_states: HashMap::new(),
            editor_ui,
            // #[cfg(feature = "plugin_shared_crate")]
            // plugins: vec![],
            #[cfg(feature = "plugin_v8")]
            v8_runtime: Some(v8_runtime),
            frame_sync,
            model_loader: ModelLoader::new(),
            window_manager: window_manager.clone(),
            material_ui_window: None,
            particle_system_ui_window: None,
            mesh_ui_window: None,
            media_ui_window: None,
            multiple_draw_ui_window: None,
            watch_shader,
            #[cfg(feature = "plugin_dotnet")]
            donet_host,
            standalone_ui_window: None,
            mosue_state: MouseState {
                is_focus: false,
                position: glam::vec2(0.0, 0.0),
            },
            player_viewport,
        };

        Ok(editor_context)
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
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
                self.standalone_ui_window = None;
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
                    .on_input(rs_engine::input_type::EInputType::MouseWheel(delta));
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
                    && *state == winit::event::ElementState::Released
                    && egui_event_response.consumed == false
                {
                    if let Some(level) = self.data_source.level.as_ref() {
                        let level = level.borrow();
                        let componenet_type = level.ray_cast_find_component(
                            &self.mosue_state.position,
                            &glam::vec2(
                                window.inner_size().width as f32,
                                window.inner_size().height as f32,
                            ),
                            self.player_viewport.camera.get_view_matrix(),
                            self.player_viewport.camera.get_projection_matrix(),
                        );
                        if let Some(componenet_type) = componenet_type {
                            use rs_engine::scene_node::EComponentType;
                            match componenet_type {
                                EComponentType::SceneComponent(_) => {}
                                EComponentType::StaticMeshComponent(componenet) => {
                                    self.editor_ui.object_property_view.selected_object =
                                        Some(ESelectedObjectType::StaticMeshComponent(componenet));
                                }
                                EComponentType::SkeletonMeshComponent(componenet) => {
                                    self.editor_ui.object_property_view.selected_object = Some(
                                        ESelectedObjectType::SkeletonMeshComponent(componenet),
                                    );
                                }
                            }
                        } else {
                            self.editor_ui.object_property_view.selected_object = None;
                        }
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
                    self.engine
                        .send_render_command(RenderCommand::BuiltinShaderChanged(changed_result))
                }
                if let Some(project_context) = &mut self.project_context {
                    if project_context.is_need_reload_plugin() {
                        match self.try_create_plugin() {
                            Ok(plugin) => {
                                if let Some(window) = &mut self.standalone_ui_window {
                                    window.reload_plugins(vec![plugin]);
                                }
                            }
                            Err(err) => log::warn!("{}", err),
                        }
                    }
                }
                let _ = self.try_load_dotnet_plugin();
                let _ = self.try_load_js_plugin();
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
                self.player_viewport
                    .on_input(rs_engine::input_type::EInputType::KeyboardInput(
                        &self.virtual_key_code_states,
                    ));

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

    pub fn handle_event(
        &mut self,
        event: &Event<ECustomEventType>,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
    ) {
        match event {
            Event::DeviceEvent { event, .. } => {
                if let Some(ui_window) = self.mesh_ui_window.as_mut() {
                    ui_window.device_event_process(event);
                }
                if let Some(ui_window) = self.multiple_draw_ui_window.as_mut() {
                    ui_window.device_event_process(event);
                }
                if let Some(ui_window) = self.particle_system_ui_window.as_mut() {
                    ui_window.device_event_process(event);
                }
                if let Some(ui_window) = self.standalone_ui_window.as_mut() {
                    ui_window.device_event_process(event);
                }
                self.player_viewport
                    .on_input(rs_engine::input_type::EInputType::Device(event));
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
                let window = &mut *window.borrow_mut();
                let egui_event_response = self.egui_winit_state.on_window_event(window, event);
                match window_type {
                    EWindowType::Main => self.main_window_event_process(
                        window_id,
                        window,
                        event,
                        event_loop_window_target,
                        egui_event_response,
                    ),
                    EWindowType::Material => {
                        let ui_window = self
                            .material_ui_window
                            .as_mut()
                            .expect("Should not be bull");
                        ui_window.window_event_process(
                            window_id,
                            window,
                            event,
                            event_loop_window_target,
                            &mut self.engine,
                            &mut *self.window_manager.borrow_mut(),
                        );
                        let Some(event) = &ui_window.material_view.event else {
                            return;
                        };
                        match event {
                            material_view::EEventType::Update(material, resolve_result) => {
                                let mut shader_code = HashMap::new();
                                let mut material_info = HashMap::new();
                                for (k, v) in resolve_result.iter() {
                                    shader_code.insert(k.clone(), v.shader_code.clone());
                                    material_info.insert(k.clone(), v.material_info.clone());
                                }
                                let handle = self.engine.create_material(shader_code);
                                let material_content = material.borrow().get_associated_material();
                                let Some(material_content) = material_content else {
                                    return;
                                };
                                let mut material_content = material_content.borrow_mut();
                                material_content.set_pipeline_handle(handle);
                                material_content.set_material_info(material_info);
                            }
                        }
                    }
                    EWindowType::Mesh => {
                        let ui_window = self.mesh_ui_window.as_mut().expect("Should not be bull");
                        ui_window.window_event_process(
                            window_id,
                            window,
                            event,
                            event_loop_window_target,
                            &mut self.engine,
                            &mut *self.window_manager.borrow_mut(),
                        );
                    }
                    EWindowType::Media => {
                        let ui_window = self.media_ui_window.as_mut().expect("Should not be bull");
                        let is_close = ui_window.window_event_process(
                            window_id,
                            window,
                            event,
                            event_loop_window_target,
                            &mut self.engine,
                            &mut *self.window_manager.borrow_mut(),
                        );
                        if is_close {
                            self.media_ui_window = None;
                        }
                    }
                    EWindowType::MultipleDraw => {
                        let ui_window = self
                            .multiple_draw_ui_window
                            .as_mut()
                            .expect("Should not be bull");
                        ui_window.window_event_process(
                            window_id,
                            window,
                            event,
                            event_loop_window_target,
                            &mut self.engine,
                            &mut *self.window_manager.borrow_mut(),
                        );
                    }
                    EWindowType::Particle => {
                        let ui_window = self
                            .particle_system_ui_window
                            .as_mut()
                            .expect("Should not be bull");
                        ui_window.window_event_process(
                            window_id,
                            window,
                            event,
                            event_loop_window_target,
                            &mut self.engine,
                            &mut *self.window_manager.borrow_mut(),
                        );
                    }
                    EWindowType::Standalone => {
                        let ui_window = self
                            .standalone_ui_window
                            .as_mut()
                            .expect("Should not be bull");
                        let mut is_close = false;
                        ui_window.window_event_process(
                            window_id,
                            window,
                            event,
                            event_loop_window_target,
                            &mut self.engine,
                            &mut *self.window_manager.borrow_mut(),
                            &mut is_close,
                        );
                        if event == &WindowEvent::CloseRequested || is_close {
                            self.standalone_ui_window = None;
                        }
                    }
                    EWindowType::Actor => todo!(),
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

    fn try_create_plugin(&mut self) -> anyhow::Result<Box<dyn Plugin>> {
        #[cfg(feature = "plugin_shared_crate")]
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
            if let Some(v8_runtime) = self.v8_runtime.as_mut() {
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
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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

    fn content_load_resources(
        engine: &mut rs_engine::engine::Engine,
        model_loader: &mut ModelLoader,
        project_context: &ProjectContext,
        files: Vec<EContentFileType>,
    ) {
        let _span = tracy_client::span!();

        let project_folder_path = project_context.get_project_folder_path();
        let asset_folder_path = project_context.get_asset_folder_path();
        for file in files {
            match file {
                EContentFileType::StaticMesh(static_mesh) => {
                    let file_path = asset_folder_path
                        // .join(&static_mesh.borrow().asset_reference_relative_path);
                        .join(&static_mesh.borrow().asset_info.relative_path);
                    model_loader.load(&file_path).unwrap();
                    let _ = model_loader.to_runtime_static_mesh(
                        &static_mesh.borrow(),
                        &asset_folder_path,
                        ResourceManager::default(),
                    );
                }
                EContentFileType::SkeletonMesh(skeleton_mesh) => {
                    let file_path =
                        project_folder_path.join(&skeleton_mesh.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    model_loader.to_runtime_skin_mesh(
                        &skeleton_mesh.borrow(),
                        &project_folder_path,
                        ResourceManager::default(),
                    );
                }
                EContentFileType::SkeletonAnimation(node_animation) => {
                    let file_path =
                        project_folder_path.join(&node_animation.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    model_loader.to_runtime_skeleton_animation(
                        node_animation.clone(),
                        &project_folder_path,
                        ResourceManager::default(),
                    );
                }
                EContentFileType::Skeleton(skeleton) => {
                    let file_path =
                        project_folder_path.join(&skeleton.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    model_loader.to_runtime_skeleton(
                        skeleton.clone(),
                        &project_folder_path,
                        ResourceManager::default(),
                    );
                }
                EContentFileType::Texture(texture_file) => {
                    let texture_file = texture_file.borrow_mut();
                    let Some(image_reference) = &texture_file.get_image_reference_path() else {
                        continue;
                    };
                    let abs_path = project_folder_path.join(image_reference);
                    let _ = engine.create_texture_from_path(&abs_path, &texture_file.url);

                    {
                        let url = texture_file.url.clone();
                        let Some(virtual_image_reference) = &texture_file.virtual_image_reference
                        else {
                            continue;
                        };
                        let path = project_context
                            .get_virtual_texture_cache_dir()
                            .join(virtual_image_reference);
                        let Ok(source) = StaticVirtualTextureSource::from_file(path, None) else {
                            continue;
                        };
                        engine.create_virtual_texture_source(url.clone(), Box::new(source));
                        log::trace!(
                            "Create virtual texture source, url: {}, {}",
                            url.as_str(),
                            virtual_image_reference
                        );
                    }
                }
                EContentFileType::Level(_) => {
                    // let mut level = level.borrow_mut();
                    // level.initialize(engine);
                }
                EContentFileType::Material(material_content) => {
                    let find = project_context
                        .project
                        .materials
                        .iter()
                        .find(|x| x.borrow().url == material_content.borrow().asset_url)
                        .cloned();
                    if let Some(material_editor) = find {
                        let mut shader_code: HashMap<MaterialOptions, String> = HashMap::new();
                        let mut material_info: HashMap<MaterialOptions, MaterialInfo> =
                            HashMap::new();
                        let mut material_editor = material_editor.borrow_mut();
                        match material_resolve::resolve(
                            &material_editor.snarl,
                            MaterialOptions::all(),
                        ) {
                            Ok(resolve_result) => {
                                for (option, result) in resolve_result {
                                    shader_code.insert(option.clone(), result.shader_code);
                                    material_info.insert(option, result.material_info);
                                }
                                {
                                    let pipeline_handle = engine.create_material(shader_code);
                                    let mut material_content = material_content.borrow_mut();
                                    material_content.set_pipeline_handle(pipeline_handle);
                                    material_content.set_material_info(material_info);
                                }
                                material_editor.set_associated_material(material_content.clone());
                            }
                            Err(err) => {
                                log::warn!("{}", err);
                            }
                        }
                    }
                }
                EContentFileType::IBL(ibl) => {
                    let result = Self::load_ibl_content_resource(engine, project_context, ibl);
                    log::trace!("{:?}", result);
                }
                EContentFileType::ParticleSystem(_) => {
                    // todo!()
                }
                EContentFileType::Sound(sound) => {
                    let sound = sound.borrow_mut();
                    let url = sound.asset_info.get_url();
                    let path = project_context
                        .get_asset_folder_path()
                        .join(&sound.asset_info.relative_path);
                    let Ok(data) = std::fs::read(path) else {
                        return;
                    };
                    let sound_resouce = rs_artifact::sound::Sound {
                        url: url.clone(),
                        sound_file_type: ESoundFileType::Unknow,
                        data,
                    };
                    let rm = ResourceManager::default();
                    rm.add_sound(url, Arc::new(sound_resouce));
                }
            }
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
        window: &mut winit::window::Window,
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
            let files = &project_context.project.content.borrow().files;
            for file in files {
                match file {
                    EContentFileType::Material(material) => {
                        let url = material.borrow().url.clone();
                        materials.push(url);
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
        let _ = self.try_load_js_plugin();
        self.data_source
            .recent_projects
            .paths
            .insert(file_path.to_path_buf());
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
            .unwrap();

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

        let content = project_context.project.content.clone();
        let mut content = content.borrow_mut();
        let mut add_files: Vec<EContentFileType> = vec![];
        for static_mesh in &load_result.static_meshes {
            add_files.push(EContentFileType::StaticMesh(static_mesh.clone()));
        }
        for skeleton_meshe in &load_result.skeleton_meshes {
            add_files.push(EContentFileType::SkeletonMesh(skeleton_meshe.clone()));
        }
        for node_animation in &load_result.node_animations {
            add_files.push(EContentFileType::SkeletonAnimation(node_animation.clone()));
        }
        if let Some(skeleton) = &load_result.skeleton {
            add_files.push(EContentFileType::Skeleton(skeleton.clone()));
        }
        Self::content_load_resources(
            &mut self.engine,
            &mut self.model_loader,
            project_context,
            add_files.clone(),
        );
        content.files.append(&mut add_files);
        let mut active_level = active_level.borrow_mut();

        Self::add_new_actors(
            &mut active_level,
            &mut self.engine,
            vec![load_result.actor.clone()],
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

    fn process_redraw_request(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
        if let Some(v8_runtime) = self.v8_runtime.as_mut() {
            let _ = v8_runtime.tick(&mut self.engine);
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
                    build_built_in_resouce_url("ShadowDepthTexture").unwrap();
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
            rs_core_minimal::file_manager::get_engine_root_dir().join("rs_editor/target/shaders");
        if !output_path.exists() {
            std::fs::create_dir(output_path.clone())
                .context(anyhow!("Can not create dir {:?}", output_path))?;
        }

        let mut compile_commands = vec![];
        for buildin_shader in buildin_shaders {
            let description = buildin_shader.get_shader_description();
            let name = buildin_shader.get_name();
            let processed_code = rs_shader_compiler::pre_process::pre_process(
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

    fn open_standalone_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
    ) {
        let Some(level) = self.data_source.level.clone() else {
            return;
        };
        let active_level = &level.borrow();
        let plugins = self.try_create_plugin().map_or(vec![], |x| vec![x]);
        let contents = self.get_all_contents();
        let ui_window = StandaloneUiWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
            plugins,
            active_level,
            contents,
        )
        .expect("Should be opened");
        self.standalone_ui_window = Some(ui_window);
    }

    fn open_particle_window(
        &mut self,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        open_material: Option<Rc<RefCell<rs_engine::content::material::Material>>>,
    ) {
        let mut ui_window = MaterialUIWindow::new(
            self.editor_ui.egui_context.clone(),
            &mut *self.window_manager.borrow_mut(),
            event_loop_window_target,
            &mut self.engine,
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

    fn open_mesh_window(
        &mut self,
        skeleton_mesh: &mut rs_engine::content::skeleton_mesh::SkeletonMesh,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
        self.model_loader.load(&file_path).unwrap();
        let skin_mesh = self.model_loader.to_runtime_skin_mesh(
            skeleton_mesh,
            &project_folder_path,
            ResourceManager::default(),
        );
        ui_window.update(&mut self.engine, &skin_mesh.vertexes, &skin_mesh.indexes);
        self.mesh_ui_window = Some(ui_window);
    }

    fn open_media_window(
        &mut self,
        media_path: impl AsRef<Path>,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
    ) {
        let _span = tracy_client::span!();

        let egui_winit_state = &mut self.egui_winit_state;

        let click_event = self
            .editor_ui
            .build(egui_winit_state.egui_ctx(), &mut self.data_source);

        self.process_top_menu_event(window, click_event.menu_event, event_loop_window_target);
        self.process_click_asset_event(click_event.click_aseet, event_loop_window_target);
        self.process_content_item_property_view_event();
        self.process_content_browser_event(
            click_event.content_browser_event,
            event_loop_window_target,
        );
        self.process_debug_texture_view_event(click_event.debug_textures_view_event);
        self.process_click_actor_event(click_event.click_actor);
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

    fn process_top_menu_event(
        &mut self,
        window: &mut winit::window::Window,
        top_menu_event: Option<top_menu::EClickEventType>,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
                if let Some(project_context) = self.project_context.as_ref() {
                    let save_status = project_context.save();
                    log::trace!("Save project: {:?}", save_status);
                }
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
            top_menu::EClickEventType::Standalone => {
                let Some(_) = self.project_context.as_mut() else {
                    return;
                };
                self.open_standalone_window(event_loop_window_target);
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
                    && texture_url.host() == Some(url::Host::Domain("ShadowDepthTexture"))
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

    fn process_click_actor_event(
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
            crate::ui::level_view::EClickEventType::Actor(actor) => {
                self.editor_ui.object_property_view.selected_object =
                    Some(ESelectedObjectType::Actor(actor));
            }
            crate::ui::level_view::EClickEventType::SceneNode(scene_node) => {
                match &scene_node.borrow().component {
                    rs_engine::scene_node::EComponentType::SceneComponent(component) => {
                        self.editor_ui.object_property_view.selected_object =
                            Some(ESelectedObjectType::SceneComponent(component.clone()));
                    }
                    rs_engine::scene_node::EComponentType::StaticMeshComponent(component) => {
                        self.editor_ui.object_property_view.selected_object =
                            Some(ESelectedObjectType::StaticMeshComponent(component.clone()));
                    }
                    rs_engine::scene_node::EComponentType::SkeletonMeshComponent(component) => {
                        self.editor_ui.object_property_view.selected_object = Some(
                            ESelectedObjectType::SkeletonMeshComponent(component.clone()),
                        );
                    }
                }
            }
            crate::ui::level_view::EClickEventType::CreateDirectionalLight => {
                let size = 10.0;
                let mut light = DirectionalLight::new(-size, size, -size, size, 0.01, 25.0);
                light.initialize(&mut self.engine, &mut self.player_viewport);
                opened_level
                    .borrow_mut()
                    .directional_lights
                    .push(SingleThreadMut::new(light));
            }
            crate::ui::level_view::EClickEventType::DirectionalLight(light) => {
                self.editor_ui.object_property_view.selected_object =
                    Some(ESelectedObjectType::DirectionalLight(light));
            }
        }
    }

    fn process_content_browser_event(
        &mut self,
        content_browser_event: Option<content_browser::EClickEventType>,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
                    EContentFileType::StaticMesh(_) => {}
                    EContentFileType::SkeletonMesh(skeleton_mesh) => {
                        self.open_mesh_window(
                            &mut *skeleton_mesh.borrow_mut(),
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
                {
                    let resolve_result = material_view::MaterialView::default_resolve().unwrap();
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
        }
    }

    fn process_click_asset_event(
        &mut self,
        click_aseet: Option<asset_view::EClickItemType>,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
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
                } else {
                    let result = self.open_model_file(asset_file.path.clone());
                    log::trace!("{:?}", result);
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
        }
    }

    fn process_content_item_property_view_event(&mut self) {
        let Some(click) = &self.editor_ui.content_item_property_view.click else {
            return;
        };
        match click {
            content_item_property_view::EClickType::IBL(ibl, old, new) => {
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
            content_item_property_view::EClickType::IsVirtualTexture(
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
            content_item_property_view::EClickType::SDF2D(texture) => {
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
        }
    }

    fn process_object_property_view_event(
        &mut self,
        event: Option<object_property_view::EEventType>,
    ) {
        let Some(event) = event else {
            return;
        };
        match event {
            object_property_view::EEventType::UpdateMaterial(update_material) => {
                match update_material.selected_object {
                    ESelectedObjectType::StaticMeshComponent(static_mesh_component) => {
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
                    ESelectedObjectType::SkeletonMeshComponent(static_mesh_component) => {
                        let files = if let Some(folder) =
                            &self.data_source.content_data_source.current_folder
                        {
                            folder.borrow().files.clone()
                        } else {
                            vec![]
                        };
                        let mut static_mesh_component = static_mesh_component.borrow_mut();
                        if let Some(url) = update_material.new {
                            static_mesh_component.set_material(
                                &mut self.engine,
                                url,
                                &files,
                                &mut self.player_viewport,
                            );
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
        let rigid_body_set = active_level
            .get_physics_mut()
            .map(|x| &mut x.rigid_body_set);

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
            ESelectedObjectType::SceneComponent(component) => {
                let mut component = component.borrow_mut();
                if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                    let parent_final_transformation = component.get_parent_final_transformation();
                    let model_matrix = component.get_transformation_mut();
                    *model_matrix =
                        parent_final_transformation.inverse() * gizmo_final_transformation;
                }
            }
            ESelectedObjectType::StaticMeshComponent(component) => {
                let mut component = component.borrow_mut();
                if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                    let parent_final_transformation = component.get_parent_final_transformation();
                    let model_matrix = component.get_transformation_mut();
                    *model_matrix =
                        parent_final_transformation.inverse() * gizmo_final_transformation;
                    component.set_apply_simulate(false);
                    component.on_post_update_transformation(rigid_body_set);
                } else {
                    component.set_apply_simulate(true);
                }
            }
            ESelectedObjectType::SkeletonMeshComponent(component) => {
                if let Some(gizmo_final_transformation) = gizmo_final_transformation {
                    let mut component = component.borrow_mut();
                    *component.get_transformation_mut() = gizmo_final_transformation;
                }
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
