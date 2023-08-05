use super::ffi::{
    acceleration_bake::AccelerationBakerJSClass, bake_info::BakeInfoJSClass, log::rs_Log_trace,
};
use rs_quickjs::{quick_js_context::QuickJSContext, quick_js_runtime::QuickJSRuntime};
use std::{cell::RefCell, sync::Arc};

pub struct QuickJSRuntimeContext {
    runtime: Arc<RefCell<QuickJSRuntime>>,
    context: QuickJSContext,
}

impl QuickJSRuntimeContext {
    pub fn new() -> QuickJSRuntimeContext {
        let runtime = Arc::new(RefCell::new(QuickJSRuntime::new()));
        runtime.borrow_mut().std_init_handlers();
        runtime.borrow_mut().set_module_loader_func();

        let context = QuickJSContext::new(runtime.clone());
        context.std_add_helpers();
        context.init_module_os();
        context.init_module_std();

        let project_description = crate::project::ProjectDescription::default();
        let project_description = project_description.lock().unwrap();
        let scripts_dir = project_description.get_paths().scripts_dir.to_string();
        let script_filename = scripts_dir + "/main.js";

        let mut js_runtime = QuickJSRuntimeContext { runtime, context };
        js_runtime.register();
        js_runtime.eval_file_module(&script_filename);
        js_runtime
    }

    pub fn eval_file_module(&self, filename: &str) {
        self.context.eval_file_module(filename);
    }

    pub fn register(&mut self) {
        let global_obj = self.context.get_global_object();
        {
            let console_obj = self.context.get_property_str(global_obj, "console");
            let c_function = self
                .context
                .new_c_function(Some(rs_Log_trace), "rs_Log_trace", 0);
            self.context
                .set_property_str(console_obj, "log", c_function);
            self.context.free_value(console_obj);
        }
        {
            let cls = AccelerationBakerJSClass::default().lock().unwrap();
            let constructor_class_func = cls.import(&mut self.context);
            self.context
                .set_property_str(global_obj, cls.get_class_name(), constructor_class_func);

            let cls = BakeInfoJSClass::default().lock().unwrap();
            let constructor_class_func = cls.import(&mut self.context);
            self.context
                .set_property_str(global_obj, cls.get_class_name(), constructor_class_func);
        }
        self.context.free_value(global_obj);
    }
}
