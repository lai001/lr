use super::ffi::log::rs_Log_trace;
use rs_quickjs::quickjs_bindings::*;

pub struct QuickJSRuntimeContext {
    runtime: QuickJSRuntime,
    context: QuickJSContext,
}

impl QuickJSRuntimeContext {
    pub fn new() -> QuickJSRuntimeContext {
        let mut runtime = QuickJSRuntime::new();
        let mut context = QuickJSContext::new(&mut runtime);
        runtime.std_init_handlers();
        runtime.set_module_loader_func();
        context.std_add_helpers();
        context.init_module_os();
        context.init_module_std();
        let mut js_runtime = QuickJSRuntimeContext {
            runtime,
            context: context,
        };
        js_runtime.register();
        let project_description = crate::project::ProjectDescription::default();
        let project_description = project_description.lock().unwrap();
        let scripts_dir = project_description.get_paths().scripts_dir.to_string();
        let filename = scripts_dir + "/main.js";
        js_runtime.eval_file_module(&filename);
        js_runtime
    }

    pub fn eval_file_module(&mut self, filename: &str) {
        self.context.eval_file_module(filename);
    }

    pub fn register(&mut self) {
        self.context.get_global_object(|context, global_obj| {
            context.get_property_str(global_obj, "console", |context, console_obj| {
                let c_function = context.new_c_function(Some(rs_Log_trace), "rs_Log_trace", 0);
                context.set_property_str(console_obj, "log", c_function);
            });
        });
    }
}
