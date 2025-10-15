use crate::{platform_wrapper::PlatformWrapper, util::log_callback, util::println_callback};
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::DebouncedEvent;
use notify_debouncer_mini::Debouncer;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use v8::Context;
use v8::Global;
use v8::OwnedIsolate;

pub struct Constructor {
    pub name: String,
    pub function_template: v8::Global<v8::FunctionTemplate>,
}

pub trait Newable: v8::cppgc::GarbageCollected + 'static {
    type AssociatedType;

    fn new(value: Self::AssociatedType) -> Self;
}

pub trait Constructible {
    type AssociatedType;

    fn construct(v8_runtime: &mut V8Runtime) -> crate::error::Result<Constructor>;
}

// const CPPGC_TAG: u16 = 1;
// const INNER_KEY: &'static str = "__inner__";
pub const V8_RUNTIME_DATA_OFFSET: u32 = 0;
pub const CPPGC_TAG: u16 = 1;

pub(crate) fn cppgc_template_constructor(
    _scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
}

pub(crate) fn make_cppgc_template<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i, ()>,
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
    constructors: HashMap<TypeId, Constructor>,
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
            v8::scope!(let handle_scope, &mut isolate);
            let local_context = v8::Context::new(handle_scope, v8::ContextOptions::default());
            let mut context_scope = v8::ContextScope::new(handle_scope, local_context);
            Global::new(&mut context_scope, local_context)
        };

        let cppgc_template = {
            v8::scope!(let handle_scope, &mut isolate);
            let cppgc_template = make_cppgc_template(handle_scope);
            let cppgc_template = RefCell::new(v8::Global::new(handle_scope, cppgc_template));
            cppgc_template
        };
        let runtime = V8Runtime {
            _platform_wrapper: platform_wrapper,
            script_watcher: None,
            isolate,
            global_context,
            gc_template: cppgc_template,
            plugins: vec![],
            constructors: HashMap::new(),
        };
        runtime
    }

    pub fn associate_embedder_specific_data(&mut self) {
        let raw_ptr: *mut V8Runtime = self as *mut _;
        self.isolate
            .set_data(V8_RUNTIME_DATA_OFFSET, raw_ptr as *mut std::ffi::c_void);
    }

    fn create_plugin(&mut self) -> crate::error::Result<v8::Global<v8::Object>> {
        v8::scope!(let handle_scope, &mut self.isolate);
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
        v8::scope!(let handle_scope, &mut self.isolate);
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
        let function = v8::Function::new(scope, log_callback).ok_or(crate::error::Error::Null(
            format!("Failed to create println function"),
        ))?;
        let name = v8::String::new(scope, "log").ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        let console = v8::Object::new(scope);
        console.set(scope, name.into(), function.into());
        let name = v8::String::new(scope, "console").ok_or(crate::error::Error::Null(format!(
            "Failed to create string"
        )))?;
        global_this.set(scope, name.into(), console.into());
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
        v8::scope!(let handle_scope, &mut self.isolate);
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
        v8::tc_scope!(let try_catch, context_scope);

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
        engine: &v8::Global<v8::Object>,
        level: &v8::Global<v8::Object>,
        player_viewport: &v8::Global<v8::Object>,
    ) -> crate::error::Result<()> {
        let plugin = self
            .get_plugin()
            .ok_or(crate::error::Error::Null(format!("No plugin")))?;
        v8::scope!(let handle_scope, &mut self.isolate);
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

    pub fn make_wrapped_value<T>(
        &mut self,
        wrapped_value: T::AssociatedType,
    ) -> crate::error::Result<v8::Global<v8::Object>>
    where
        T: Newable,
    {
        let type_name = std::any::type_name::<T>();
        let type_id = std::any::TypeId::of::<T>();
        let function_template = &self
            .constructors
            .get(&type_id)
            .expect(&format!(
                "{} constructor missing. Register it before use.",
                type_name
            ))
            .function_template;
        let main_context = self.global_context.clone();
        let isolate = &mut self.isolate;
        v8::scope_with_context!(context_scope, isolate, &main_context);
        let scope = context_scope;
        let local_function = v8::Local::new(scope, function_template);
        let function = local_function
            .get_function(scope)
            .ok_or(crate::error::Error::Null(format!("Function is null")))?;
        let obj = function.new_instance(scope, &[]).expect("Not null");
        unsafe {
            let native_value: T = T::new(wrapped_value.into());
            let member: v8::cppgc::UnsafePtr<T> = v8::cppgc::make_garbage_collected::<T>(
                scope.get_cpp_heap().expect("Not null"),
                native_value,
            );
            v8::Object::wrap::<{ CPPGC_TAG }, T>(scope, obj, &member);
        }
        Ok(v8::Global::new(scope, obj))
    }

    pub fn register_constructors(&mut self, constructors: HashMap<TypeId, Constructor>) {
        self.constructors.extend(constructors);
    }

    pub fn register_constructor<T: Constructible + 'static>(&mut self) {
        let k = std::any::TypeId::of::<T::AssociatedType>();
        match T::construct(self) {
            Ok(constructor) => {
                self.constructors.insert(k, constructor);
            }
            Err(err) => {
                let name = std::any::type_name::<T::AssociatedType>();
                log::warn!("{}, failed to create constructor, {err}", name);
            }
        }
    }

    pub fn register_constructor2<T: 'static>(&mut self, constructor: Constructor) {
        let k = std::any::TypeId::of::<T>();
        self.constructors.insert(k, constructor);
    }
}

impl Drop for V8Runtime {
    fn drop(&mut self) {
        // unsafe { ManuallyDrop::drop(&mut self.isolate) };
    }
}
