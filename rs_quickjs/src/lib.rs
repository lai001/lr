pub mod quickjs_bindings;

#[cfg(test)]
mod tests {
    use super::quickjs_bindings::*;
    use std::ffi::CString;

    #[test]
    fn quickjs_test() {
        unsafe {
            let runtime = JS_NewRuntime();
            let context = JS_NewContext(runtime);
            assert_ne!(runtime, std::ptr::null_mut());
            assert_ne!(context, std::ptr::null_mut());
            js_std_init_handlers(runtime);

            JS_SetModuleLoaderFunc(runtime, None, Some(js_module_loader), std::ptr::null_mut());
            js_std_add_helpers(context, 0, std::ptr::null_mut());
            js_init_module_std(context, "std".as_ptr() as *const i8);
            js_init_module_os(context, "os".as_ptr() as *const i8);

            js_std_free_handlers(runtime);
            JS_FreeContext(context);
            JS_FreeRuntime(runtime);
        };
    }
}
