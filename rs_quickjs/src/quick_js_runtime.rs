use crate::quickjs_bindings::*;

pub struct QuickJSRuntime {
    pub(crate) inner: *mut JSRuntime,
    is_std_init_handlers: bool,
}

impl QuickJSRuntime {
    pub fn new() -> QuickJSRuntime {
        unsafe {
            let runtime = JS_NewRuntime();
            QuickJSRuntime {
                inner: runtime,
                is_std_init_handlers: false,
            }
        }
    }
    pub fn std_init_handlers(&mut self) {
        self.is_std_init_handlers = true;
        unsafe {
            js_std_init_handlers(self.inner);
        }
    }

    pub fn std_free_handlers(&mut self) {
        if self.is_std_init_handlers {
            unsafe {
                js_std_free_handlers(self.inner);
            }
            self.is_std_init_handlers = false;
        }
    }

    pub fn set_module_loader_func(&self) {
        unsafe {
            JS_SetModuleLoaderFunc(
                self.inner,
                None,
                Some(js_module_loader),
                std::ptr::null_mut(),
            );
        }
    }

    pub fn new_class(&self, class_id: JSClassID, class_def: &JSClassDef) {
        unsafe {
            let state = JS_NewClass(self.inner, class_id, class_def);
            assert_eq!(state, 0);
        }
    }
}

impl Drop for QuickJSRuntime {
    fn drop(&mut self) {
        self.std_free_handlers();
        unsafe {
            JS_FreeRuntime(self.inner);
        }
    }
}
