use crate::{
    custom_event::ECustomEventType,
    editor_context::{EWindowType, EditorContext},
    windows_manager::WindowsManager,
};
use anyhow::anyhow;
use clap::*;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use rs_engine::logger::SlotFlags;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event_loop::EventLoop,
    platform::windows::EventLoopBuilderExtWindows,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    cmd: bool,
    #[arg(long)]
    input_file: Option<std::path::PathBuf>,
    #[arg(short, long)]
    definitions: Option<Vec<String>>,
    #[arg(long)]
    include_dirs: Option<Vec<std::path::PathBuf>>,
    #[arg(short, long)]
    output_file: Option<std::path::PathBuf>,
}

pub struct Editor {}

impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(self) -> anyhow::Result<()> {
        let result = self.run_internal();
        #[cfg(feature = "exit_check")]
        let _ = std::io::stdin().read_line(&mut String::new());
        result
    }

    fn run_internal(mut self) -> anyhow::Result<()> {
        let is_run_app = self.parse_args();
        if is_run_app {
            self.run_app()?;
        }
        Ok(())
    }

    fn parse_args(&mut self) -> bool {
        let args = Args::parse();
        if !args.cmd {
            return true;
        }
        rs_foundation::change_working_directory();
        let _ = rs_engine::logger::Logger::new(rs_engine::logger::LoggerConfiguration {
            is_write_to_file: false,
            is_flush_before_drop: true,
            slot_flags: SlotFlags::empty(),
        });
        match args.input_file {
            Some(input_file) => {
                let result: anyhow::Result<String> = (|| {
                    let include_dirs = args.include_dirs.unwrap_or(vec![]);
                    let definitions = args.definitions.unwrap_or(vec![]);

                    let result = rs_shader_compiler::pre_process::pre_process(
                        &input_file,
                        include_dirs.iter(),
                        definitions.iter(),
                    )?;
                    let _ = naga::front::wgsl::parse_str(&result)?;
                    match args.output_file {
                        Some(output_file) => {
                            let _ = std::fs::write(output_file, result.clone())?;
                        }
                        None => {}
                    }
                    Ok(result)
                })();
                match result {
                    Ok(result) => log::trace!("{}", result),
                    Err(err) => log::error!("{}", err),
                }
            }
            None => {
                let result: anyhow::Result<()> = (|| {
                    EditorContext::prepreocess_shader()?;
                    let output_path = rs_core_minimal::file_manager::get_engine_root_dir()
                        .join("rs_editor/target/shaders");
                    for entry in walkdir::WalkDir::new(output_path) {
                        let entry = entry?;
                        if !entry.path().is_file() {
                            continue;
                        }
                        let path = entry.path();
                        let path = std::env::current_dir()?.join(path).canonicalize_slash()?;
                        println!("{:?}", &path);
                        let shader_source = std::fs::read_to_string(path)?;
                        naga::front::wgsl::parse_str(&shader_source)?;
                    }
                    Ok(())
                })();
                match result {
                    Ok(_) => {}
                    Err(err) => log::error!("{}", err),
                }
            }
        }
        return false;
    }

    fn run_app(self) -> anyhow::Result<()> {
        let window_manager = SingleThreadMut::new(WindowsManager::new());
        let event_loop = EventLoop::<ECustomEventType>::with_user_event()
            .with_any_thread(true)
            .build()?;
        let event_loop_proxy = event_loop.create_proxy();
        let mut app = EditorApplicationHandler {
            editor_context: None,
            window_manager,
            event_loop_proxy,
        };
        let event_loop_result = event_loop.run_app(&mut app);
        Ok(event_loop_result?)
    }

    pub fn default_icon() -> anyhow::Result<winit::window::Icon> {
        let icon_image = image::load_from_memory(include_bytes!("../target/editor.ico"))?;
        let icon_image = icon_image.as_rgba8().ok_or(anyhow!("Bad icon"))?;
        let icon = winit::window::Icon::from_rgba(
            icon_image.to_vec(),
            icon_image.width(),
            icon_image.height(),
        )?;
        Ok(icon)
    }
}

struct EditorApplicationHandler {
    editor_context: Option<EditorContext>,
    window_manager: SingleThreadMutType<WindowsManager>,
    event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
}

impl ApplicationHandler<ECustomEventType> for EditorApplicationHandler {
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        match &cause {
            winit::event::StartCause::Init => {
                let scale_factor = event_loop
                    .primary_monitor()
                    .map(|x| x.scale_factor())
                    .unwrap_or(1.0);
                let window_width = (1280 as f64 * scale_factor) as u32;
                let window_height = (720 as f64 * scale_factor) as u32;
                let window_attributes = winit::window::Window::default_attributes()
                    .with_window_icon(Editor::default_icon().ok())
                    .with_decorations(true)
                    .with_resizable(true)
                    .with_transparent(false)
                    .with_inner_size(PhysicalSize::new(window_width, window_height))
                    .with_title("Editor");
                let window = event_loop
                    .create_window(window_attributes)
                    .expect("Should not be null");
                window.set_ime_allowed(true);
                let editor_context = EditorContext::new(
                    u64::from(window.id()) as isize,
                    &window,
                    self.event_loop_proxy.clone(),
                    self.window_manager.clone(),
                )
                .expect("Should not be null");
                self.editor_context = Some(editor_context);
                self.window_manager
                    .borrow_mut()
                    .add_new_window(EWindowType::Main, window);
            }
            _ => {}
        }
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: ECustomEventType,
    ) {
        let event = winit::event::Event::UserEvent(event);
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let event = winit::event::Event::DeviceEvent { device_id, event };
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let event = winit::event::Event::AboutToWait;
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let event = winit::event::Event::Suspended;
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let event = winit::event::Event::LoopExiting;
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let event = winit::event::Event::MemoryWarning;
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let event = winit::event::Event::Resumed;
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let event = winit::event::Event::WindowEvent { window_id, event };
        self.editor_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(&event, event_loop);
    }
}
