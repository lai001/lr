use crate::{platform_wrapper::PlatformWrapper, util::println_callback};
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::DebouncedEvent;
use notify_debouncer_mini::Debouncer;
use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use v8::Context;
use v8::Global;
use v8::OwnedIsolate;

// const CPPGC_TAG: u16 = 1;
// const INNER_KEY: &'static str = "__inner__";
pub const V8_RUNTIME_DATA_OFFSET: u32 = 0;
pub const CPPGC_TAG: u16 = 1;

pub(crate) fn cppgc_template_constructor(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
}

pub(crate) fn make_cppgc_template<'s>(
    scope: &mut v8::HandleScope<'s, ()>,
) -> v8::Local<'s, v8::FunctionTemplate> {
    v8::FunctionTemplate::new(scope, cppgc_template_constructor)
}

struct ScriptWatcher {
    receiver: Receiver<std::result::Result<Vec<DebouncedEvent>, notify::Error>>,
    _debouncer: Debouncer<ReadDirectoryChangesWatcher>,
    _watch_dir: PathBuf,
    _watch_entry_file: PathBuf,
}

pub struct V8Runtime {
    pub isolate: OwnedIsolate,
    _platform_wrapper: PlatformWrapper,
    script_watcher: Option<ScriptWatcher>,
    pub global_context: Global<Context>,
    pub gc_template: RefCell<v8::Global<v8::FunctionTemplate>>,
    pub plugins: Vec<v8::Global<v8::Object>>, // pub isolate: ManuallyDrop<OwnedIsolate>,
}

impl V8Runtime {
    pub fn new() -> V8Runtime {
        let platform_wrapper = PlatformWrapper::new();

        let heap = v8::cppgc::Heap::create(
            platform_wrapper.platform.clone(),
            v8::cppgc::HeapCreateParams::default(),
        );
        let mut isolate = v8::Isolate::new(v8::CreateParams::default().cpp_heap(heap));
        // let isolate =
        //     ManuallyDrop::new(v8::Isolate::new(v8::CreateParams::default().cpp_heap(heap)));

        let global_context = {
            let isolate = &mut isolate;
            let mut handle_scope = v8::HandleScope::new(isolate);
            let local_context = v8::Context::new(&mut handle_scope, v8::ContextOptions::default());
            let mut context_scope = v8::ContextScope::new(&mut handle_scope, local_context);
            Global::new(&mut context_scope, local_context)
        };

        let cppgc_template = {
            let mut handle_scope = v8::HandleScope::new(&mut isolate);
            let cppgc_template = make_cppgc_template(&mut handle_scope);
            let cppgc_template = RefCell::new(v8::Global::new(&mut handle_scope, cppgc_template));
            cppgc_template
        };
        let runtime = V8Runtime {
            _platform_wrapper: platform_wrapper,
            script_watcher: None,
            isolate,
            global_context,
            gc_template: cppgc_template,
            plugins: vec![],
        };
        runtime
    }

    pub fn associate_embedder_specific_data(&mut self) {
        let raw_ptr: *mut V8Runtime = self as *mut _;
        let mut context_scope: v8::HandleScope =
            v8::HandleScope::with_context(&mut self.isolate, &self.global_context);
        let scope = &mut context_scope;
        scope.set_data(V8_RUNTIME_DATA_OFFSET, raw_ptr as *mut std::ffi::c_void);
    }

    fn create_plugin(&mut self) -> crate::error::Result<v8::Global<v8::Object>> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let global_this = context.global(scope);
        let name = v8::String::new(scope, "createPlugin").ok_or(crate::error::Error::Null(
            format!("Failed to create string"),
        ))?;
        let create_plugin_function = global_this
            .get(scope, name.into())
            .map(|x| v8::Local::<v8::Function>::try_from(x).ok())
            .flatten()
            .ok_or(crate::error::Error::Null(format!("No method")))?;
        let plugin = create_plugin_function
            .call(scope, global_this.into(), &[])
            .ok_or_else(|| crate::error::Error::Other(format!("Failed to create plugin")))?;
        if !plugin.is_object() {
            return Err(crate::error::Error::Other(format!("Expect an object")));
        }
        let plugin: v8::Local<v8::Object> = plugin.cast();
        let plugin = v8::Global::new(scope, plugin);

        Ok(plugin)
    }

    fn get_plugin(&self) -> Option<v8::Global<v8::Object>> {
        self.plugins.last().cloned()
    }

    pub fn register_func_global(&mut self) -> crate::error::Result<()> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let function = v8::Function::new(scope, println_callback).ok_or(
            crate::error::Error::Null(format!("Failed to create println function")),
        )?;
        let global_this = context.global(scope);
        let name = v8::String::new(scope, "println").ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        global_this.set(scope, name.into(), function.into());
        Ok(())
    }

    // pub fn register_func(&mut self) -> crate::error::Result<()> {
    //     let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
    //     let context = v8::Context::new(handle_scope);
    //     let scope = &mut v8::ContextScope::new(handle_scope, context);
    //     let function = v8::Function::new(scope, println_callback).ok_or(
    //         crate::error::Error::Null(format!("Failed to create println function")),
    //     )?;
    //     let global_this = context.global(scope);
    //     let name = v8::String::new(scope, "println").ok_or(crate::error::Error::Null(format!(
    //         "Failed to create string"
    //     )))?;
    //     global_this.set(scope, name.into(), function.into());
    //     Ok(())
    // }

    pub fn execute_script_file_global(
        &mut self,
        path: impl AsRef<Path>,
    ) -> crate::error::Result<()> {
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|err| crate::error::Error::IO(err, None))?;
        self.execute_script_code(source)?;
        match self.create_plugin() {
            Ok(plugin) => {
                self.plugins.push(plugin);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn execute_script_code(&mut self, source: String) -> crate::error::Result<()> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let global_context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, global_context);
        let source = v8::String::new(scope, &source).ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        Self::execute_script(scope, source)
    }

    fn execute_script(
        context_scope: &mut v8::ContextScope<v8::HandleScope>,
        script: v8::Local<v8::String>,
    ) -> crate::error::Result<()> {
        let mut scope = v8::HandleScope::new(context_scope);
        let try_catch = &mut v8::TryCatch::new(&mut scope);

        let script = v8::Script::compile(try_catch, script, None).ok_or(
            crate::error::Error::Other(format!("Failed to compile script")),
        )?;

        if script.run(try_catch).is_none() {
            let exception_string = try_catch
                .stack_trace()
                .or_else(|| try_catch.exception())
                .map(|value| value.to_rust_string_lossy(try_catch))
                .unwrap_or_else(|| "No stack trace".into());
            return Err(crate::error::Error::Execute(exception_string));
        }

        Ok(())
    }

    pub fn reload_script(&mut self) -> crate::error::Result<()> {
        let watch_path = self
            .script_watcher
            .as_mut()
            .map(|x| x._watch_entry_file.clone())
            .ok_or(crate::error::Error::Other(format!("Did not start watch")))?;
        self.execute_script_file_global(watch_path)
    }

    pub fn start_watch(
        &mut self,
        watch_dir: impl AsRef<Path>,
        watch_entry_file: impl AsRef<Path>,
    ) -> crate::error::Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut debouncer =
            notify_debouncer_mini::new_debouncer(std::time::Duration::from_millis(1000), sender)
                .map_err(|err| crate::error::Error::Debouncer(err))?;

        debouncer
            .watcher()
            .watch(watch_dir.as_ref(), notify::RecursiveMode::Recursive)
            .map_err(|err| crate::error::Error::Debouncer(err))?;
        let script_watcher = ScriptWatcher {
            receiver,
            _debouncer: debouncer,
            _watch_dir: watch_dir.as_ref().to_path_buf(),
            _watch_entry_file: watch_entry_file.as_ref().to_path_buf(),
        };
        self.script_watcher = Some(script_watcher);
        Ok(())
    }

    pub fn is_need_reload(&self) -> bool {
        if let Some(watcher) = self.script_watcher.as_ref() {
            let mut is_need_reload = false;
            for events in watcher.receiver.try_iter().filter(|x| x.is_ok()).flatten() {
                if !is_need_reload {
                    is_need_reload = events.is_empty() == false;
                }
            }
            return is_need_reload;
        } else {
            return false;
        }
    }

    pub fn is_watching(&self) -> bool {
        self.script_watcher.is_some()
    }

    pub fn tick(
        &mut self,
        engine: v8::Global<v8::Object>,
        level: v8::Global<v8::Object>,
        player_viewport: v8::Global<v8::Object>,
    ) -> crate::error::Result<()> {
        let plugin = self
            .get_plugin()
            .ok_or(crate::error::Error::Null(format!("No plugin")))?;
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let global_this = context.global(scope);
        let plugin = v8::Local::new(scope, plugin);
        let name = v8::String::new(scope, "tick").ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        let tick = plugin
            .get(scope, name.into())
            .map(|x| v8::Local::<v8::Function>::try_from(x).ok())
            .flatten()
            .ok_or(crate::error::Error::Null(format!("No method")))?;
        let parameters = [
            v8::Local::new(scope, engine).into(),
            v8::Local::new(scope, level).into(),
            v8::Local::new(scope, player_viewport).into(),
        ];
        tick.call(scope, global_this.into(), &parameters);
        Ok(())
    }
}

impl Drop for V8Runtime {
    fn drop(&mut self) {
        // unsafe { ManuallyDrop::drop(&mut self.isolate) };
    }
}
