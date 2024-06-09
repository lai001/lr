use crate::{
    custom_event::ECustomEventType,
    editor_context::{EWindowType, EditorContext},
};
use clap::*;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use std::collections::HashMap;
use winit::event_loop::{EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};

pub struct WindowContext {
    pub window_type: EWindowType,
    pub window: SingleThreadMutType<winit::window::Window>,
}

impl WindowContext {
    pub fn get_id(&self) -> isize {
        u64::from(self.window.borrow().id()) as isize
    }

    pub fn get_width(&self) -> u32 {
        self.window.borrow().inner_size().width
    }

    pub fn get_height(&self) -> u32 {
        self.window.borrow().inner_size().height
    }
}

pub struct WindowsManager {
    pub window_contexts: HashMap<EWindowType, WindowContext>,
}

impl WindowsManager {
    pub fn new() -> WindowsManager {
        WindowsManager {
            window_contexts: HashMap::new(),
        }
    }

    pub fn add_new_window(&mut self, window_type: EWindowType, window: winit::window::Window) {
        self.window_contexts.insert(
            window_type,
            WindowContext {
                window_type,
                window: SingleThreadMut::new(window),
            },
        );
    }

    pub fn get_window_context_by_id(&self, window_id: isize) -> Option<&WindowContext> {
        let context = self
            .window_contexts
            .iter()
            .find(|x| x.1.get_id() == window_id);
        match context {
            Some(context) => Some(context.1),
            None => None,
        }
    }

    pub fn get_window_by_id(
        &self,
        window_id: isize,
    ) -> Option<SingleThreadMutType<winit::window::Window>> {
        let context = self
            .window_contexts
            .iter()
            .find(|x| x.1.get_id() == window_id);
        match context {
            Some(context) => Some(context.1.window.clone()),
            None => None,
        }
    }

    pub fn get_window_type_by_id(&self, window_id: isize) -> Option<EWindowType> {
        let context = self
            .window_contexts
            .iter()
            .find(|x| x.1.get_id() == window_id);
        match context {
            Some(context) => Some(context.1.window_type),
            None => None,
        }
    }

    pub fn get_main_window(&self) -> SingleThreadMutType<winit::window::Window> {
        self.window_contexts
            .get(&EWindowType::Main)
            .expect("Not null")
            .window
            .clone()
    }

    pub fn spwan_new_window(
        &mut self,
        window_type: EWindowType,
        event_loop_window_target: &EventLoopWindowTarget<ECustomEventType>,
    ) -> anyhow::Result<&mut WindowContext> {
        let window_width = 1280;
        let window_height = 720;
        let child_window_builder = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title(format!("{:?}", window_type))
            .with_inner_size(winit::dpi::PhysicalSize {
                width: window_width,
                height: window_height,
            });
        let child_window =
            SingleThreadMut::new(child_window_builder.build(event_loop_window_target)?);
        self.window_contexts.insert(
            window_type,
            WindowContext {
                window_type,
                window: child_window,
            },
        );

        Ok(self
            .window_contexts
            .get_mut(&window_type)
            .ok_or(anyhow::anyhow!(""))?)
    }

    pub fn remove_window(&mut self, window_type: EWindowType) {
        self.window_contexts.remove(&window_type);
    }
}

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

    pub fn run(mut self) -> anyhow::Result<()> {
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
                let result = EditorContext::prepreocess_shader();
                let output_path = rs_core_minimal::file_manager::get_engine_root_dir()
                    .join("rs_editor/target/shaders");
                for entry in walkdir::WalkDir::new(output_path) {
                    let entry = entry.unwrap();
                    if !entry.path().is_file() {
                        continue;
                    }
                    let path = entry.path();
                    let path = std::env::current_dir()
                        .unwrap()
                        .join(path)
                        .canonicalize_slash()
                        .unwrap();
                    log::trace!("{:?}", &path);
                    let shader_source = std::fs::read_to_string(path).unwrap();
                    let result = naga::front::wgsl::parse_str(&shader_source);
                    let _ = result.unwrap();
                }
                let _ = result.unwrap();
            }
        }
        return false;
    }

    fn run_app(self) -> anyhow::Result<()> {
        let window_manager = SingleThreadMut::new(WindowsManager::new());

        let window_width = 1280;
        let window_height = 720;
        let event_loop = EventLoopBuilder::with_user_event().build()?;
        let event_loop_proxy: EventLoopProxy<ECustomEventType> = event_loop.create_proxy();

        let window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Editor")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: window_width,
                height: window_height,
            })
            .build(&event_loop)?;
        window.set_ime_allowed(true);

        let mut editor_context =
            EditorContext::new(&window, event_loop_proxy.clone(), window_manager.clone())?;
        window_manager
            .borrow_mut()
            .add_new_window(EWindowType::Main, window);

        let event_loop_result = event_loop.run({
            move |event, event_loop_window_target| {
                editor_context.handle_event(&event, event_loop_window_target);
            }
        });
        Ok(event_loop_result?)
    }
}
