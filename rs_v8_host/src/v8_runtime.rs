use crate::engine::{engine_get_camera_mut, engine_set_view_mode, NativeEngine};
use crate::{platform_wrapper::PlatformWrapper, util::println_callback};
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::DebouncedEvent;
use notify_debouncer_mini::Debouncer;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use v8::Context;
use v8::Global;
use v8::OwnedIsolate;

// const CPPGC_TAG: u16 = 1;
// const INNER_KEY: &'static str = "__inner__";

struct ScriptWatcher {
    receiver: Receiver<std::result::Result<Vec<DebouncedEvent>, notify::Error>>,
    _debouncer: Debouncer<ReadDirectoryChangesWatcher>,
    _watch_dir: PathBuf,
    _watch_entry_file: PathBuf,
}

pub struct V8Runtime {
    pub(crate) isolate: OwnedIsolate,
    pub(crate) _platform_wrapper: PlatformWrapper,
    script_watcher: Option<ScriptWatcher>,
    global_context: Global<Context>,
    // pub isolate: ManuallyDrop<OwnedIsolate>,
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
            let local_context = v8::Context::new(&mut handle_scope);
            let mut context_scope = v8::ContextScope::new(&mut handle_scope, local_context);
            Global::new(&mut context_scope, local_context)
        };

        V8Runtime {
            _platform_wrapper: platform_wrapper,
            script_watcher: None,
            isolate,
            global_context,
        }
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
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        // let context = v8::Context::new(handle_scope);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|err| crate::error::Error::IO(err, None))?;
        let source = v8::String::new(scope, &source).ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        Self::execute_script(scope, source)
    }

    pub fn execute_script_code(&mut self, source: String) -> crate::error::Result<()> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        // let context = v8::Context::new(handle_scope);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
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

    pub fn register_engine_global(
        &mut self,
        engine: &mut rs_engine::engine::Engine,
    ) -> crate::error::Result<()> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let native_engine = unsafe { NativeEngine::new(engine) };
        let native_engine = Box::new(native_engine);
        let native_engine = Box::into_raw(native_engine);

        let engine_object_template = v8::ObjectTemplate::new(scope);
        engine_object_template.set_internal_field_count(1);
        let engine_object =
            engine_object_template
                .new_instance(scope)
                .ok_or(crate::error::Error::Other(format!(
                    "Failed to create object"
                )))?;

        engine_object.set_aligned_pointer_in_internal_field(0, native_engine as _);

        let name = v8::String::new(scope, "setViewMode").ok_or(crate::error::Error::Null(
            format!("Failed to create string"),
        ))?;
        let function = v8::Function::new(scope, engine_set_view_mode).ok_or(
            crate::error::Error::Null(format!("Failed to create function")),
        )?;
        engine_object.set(scope, name.into(), function.into());

        let global_this = context.global(scope);
        let name = v8::String::new(scope, "engine").ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        global_this.set(scope, name.into(), engine_object.into());
        Ok(())
    }

    // fn empty(_: &mut v8::HandleScope, _: v8::FunctionCallbackArguments, _: v8::ReturnValue) {}

    pub fn tick(&mut self, engine: &mut rs_engine::engine::Engine) -> crate::error::Result<()> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(handle_scope, self.global_context.clone());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let global_this = context.global(scope);
        let name = v8::String::new(scope, "engineTick").ok_or(crate::error::Error::Null(
            format!("Failed to create string"),
        ))?;
        let tick = global_this
            .get(scope, name.into())
            .map(|x| v8::Local::<v8::Function>::try_from(x).ok())
            .flatten()
            .ok_or(crate::error::Error::Null(format!("No method")))?;

        let native_engine = unsafe { NativeEngine::new(engine) };
        let native_engine = Box::new(native_engine);
        let native_engine = Box::into_raw(native_engine);

        let engine_object_template = v8::ObjectTemplate::new(scope);
        engine_object_template.set_internal_field_count(1);
        let name = v8::String::new(scope, "getCameraMut").ok_or(crate::error::Error::Null(
            format!("Failed to create string"),
        ))?;
        let function = v8::FunctionTemplate::new(scope, engine_get_camera_mut);
        engine_object_template.set(name.into(), function.into());

        let name = v8::String::new(scope, "setViewMode").ok_or(crate::error::Error::Null(
            format!("Failed to create string"),
        ))?;
        let function = v8::FunctionTemplate::new(scope, engine_set_view_mode);
        engine_object_template.set(name.into(), function.into());

        let engine_object =
            engine_object_template
                .new_instance(scope)
                .ok_or(crate::error::Error::Other(format!(
                    "Failed to create object"
                )))?;

        engine_object.set_aligned_pointer_in_internal_field(0, native_engine as _);

        tick.call(scope, global_this.into(), &[engine_object.into()]);
        Ok(())
    }
}

impl Drop for V8Runtime {
    fn drop(&mut self) {
        // unsafe { ManuallyDrop::drop(&mut self.isolate) };
    }
}
