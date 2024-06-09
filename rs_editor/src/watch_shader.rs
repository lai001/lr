use anyhow::Result;
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use rs_render::{command::BuiltinShaderChanged, global_shaders::global_shader::GlobalShader};
use std::sync::mpsc::Receiver;

pub struct WatchShader {
    receiver: Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
    _debouncer: Debouncer<ReadDirectoryChangesWatcher>,
    buildin_shaders: Vec<Box<dyn GlobalShader>>,
}

impl WatchShader {
    pub fn new(shader_folder_path: impl AsRef<std::path::Path>) -> Result<WatchShader> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(std::time::Duration::from_millis(200), None, sender)?;

        debouncer.watcher().watch(
            shader_folder_path.as_ref(),
            notify::RecursiveMode::Recursive,
        )?;
        let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();

        Ok(WatchShader {
            receiver,
            _debouncer: debouncer,
            buildin_shaders,
        })
    }

    pub fn get_changed_results(&self) -> Vec<BuiltinShaderChanged> {
        let mut builtin_shader_changeds = vec![];
        if !rs_core_minimal::misc::is_run_from_ide() {
            return builtin_shader_changeds;
        }
        let events: Vec<DebouncedEvent> = self.receiver.try_iter().flatten().flatten().collect();
        for event in events {
            for buildin_shader in &self.buildin_shaders {
                let description = buildin_shader.get_shader_description();
                let name = buildin_shader.get_name();
                if description.shader_path != event.path {
                    continue;
                }
                let pre_process_code = rs_shader_compiler::pre_process::pre_process(
                    &description.shader_path,
                    description.include_dirs.iter(),
                    description.definitions.iter(),
                );
                match pre_process_code {
                    Ok(source) => {
                        builtin_shader_changeds.push(BuiltinShaderChanged { name, source });
                        continue;
                    }
                    Err(err) => {
                        log::trace!("{err}");
                    }
                }
            }
        }

        builtin_shader_changeds
    }
}
