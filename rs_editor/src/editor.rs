use crate::{custom_event::ECustomEventType, editor_context::EditorContext};
use clap::*;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use winit::event_loop::{EventLoopBuilder, EventLoopProxy};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    prepreocess_shader: bool,
}

pub struct Editor {}

impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(mut self) {
        let is_run_app = self.parse_args();
        if is_run_app {
            self.run_app();
        }
    }

    fn parse_args(&mut self) -> bool {
        rs_foundation::change_working_directory();
        let args = Args::parse();
        if args.prepreocess_shader {
            let result = EditorContext::prepreocess_shader();
            let output_path = rs_core_minimal::file_manager::get_engine_root_dir()
                .join("rs_editor/target/shaders");
            for entry in walkdir::WalkDir::new(output_path) {
                let entry = entry.unwrap();
                if entry.path().is_file() {
                    let path = entry.path();
                    println!(
                        "{:?}",
                        std::env::current_dir()
                            .unwrap()
                            .join(path)
                            .canonicalize_slash()
                    );
                    let shader_source = std::fs::read_to_string(path).unwrap();
                    let result = naga::front::wgsl::parse_str(&shader_source);
                    let _ = result.unwrap();
                }
            }
            let _ = result.unwrap();
            return false;
        }
        true
    }

    fn run_app(self) {
        let window_width = 1280;
        let window_height = 720;
        let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
        let event_loop_proxy: EventLoopProxy<ECustomEventType> = event_loop.create_proxy();
        let mut window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Editor")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: window_width,
                height: window_height,
            })
            .build(&event_loop)
            .unwrap();
        window.set_ime_allowed(true);
        let mut editor_context = EditorContext::new(&window, event_loop_proxy.clone());
        let event_loop_result = event_loop.run({
            move |event, event_loop_window_target| {
                editor_context.handle_event(&mut window, &event, event_loop_window_target);
            }
        });
        match event_loop_result {
            Ok(_) => {}
            Err(err) => {
                log::warn!("{}", err);
            }
        }
    }
}
