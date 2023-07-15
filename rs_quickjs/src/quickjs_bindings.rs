#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type QuickJSValue = JSValue;
pub type QuickJSCFunction = JSCFunction;

pub struct QuickJSContext {
    inner: *mut JSContext,
}

impl QuickJSContext {
    pub fn new(runtime: &mut QuickJSRuntime) -> QuickJSContext {
        unsafe {
            let context = JS_NewContext(runtime.inner);

            QuickJSContext { inner: context }
        }
    }

    pub fn std_add_helpers(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        let mut cstr_argv: Vec<_> = args
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap())
            .collect();
        let mut p_argv: Vec<_> = cstr_argv
            .iter_mut()
            .map(|arg| arg.as_ptr() as *mut std::os::raw::c_char)
            .collect();
        p_argv.push(std::ptr::null_mut());

        unsafe {
            js_std_add_helpers(self.inner, args.len() as i32, p_argv.as_mut_ptr());
        }
    }

    pub fn init_module_std(&mut self) {
        unsafe {
            js_init_module_std(self.inner, "std".as_ptr() as *const i8);
        }
    }

    pub fn init_module_os(&mut self) {
        unsafe {
            js_init_module_os(self.inner, "os".as_ptr() as *const i8);
        }
    }

    pub fn eval_file_module(&mut self, filename: &str) {
        unsafe {
            let c = CString::new(filename).unwrap();
            QuickjsHelper_evalFile(
                self.inner,
                c.as_ptr() as *const i8,
                JS_EVAL_TYPE_MODULE as i32,
            );
        }
    }

    pub fn get_global_object(
        &mut self,
        mut closure: impl FnMut(&mut QuickJSContext, JSValue) -> (),
    ) {
        unsafe {
            let global_object = JS_GetGlobalObject(self.inner);
            closure(self, global_object);
            self.free_value(global_object)
        }
    }

    pub fn free_value(&mut self, value: JSValue) {
        unsafe {
            QuickJS_FreeValue(self.inner, value);
        }
    }

    pub fn set_property_str(&self, this_obj: JSValue, prop: &str, val: JSValue) {
        let c = CString::new(prop).unwrap();
        unsafe {
            JS_SetPropertyStr(self.inner, this_obj, c.as_ptr(), val);
        }
    }

    pub fn new_c_function(&self, func: JSCFunction, name: &str, length: i32) -> JSValue {
        let c = CString::new(name).unwrap();
        unsafe { QuickJS_NewCFunction(self.inner, func, c.as_ptr() as *const i8, length) }
    }

    pub fn get_property_str(
        &mut self,
        this_obj: JSValue,
        name: &str,
        mut closure: impl FnMut(&mut QuickJSContext, JSValue) -> (),
    ) {
        let c = CString::new(name).unwrap();
        unsafe {
            let object = JS_GetPropertyStr(self.inner, this_obj, c.as_ptr());
            closure(self, object);
            self.free_value(object)
        }
    }
}

impl Drop for QuickJSContext {
    fn drop(&mut self) {
        unsafe {
            JS_FreeContext(self.inner);
        }
    }
}

pub struct QuickJSRuntime {
    inner: *mut JSRuntime,
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

    pub fn set_module_loader_func(&mut self) {
        unsafe {
            JS_SetModuleLoaderFunc(
                self.inner,
                None,
                Some(js_module_loader),
                std::ptr::null_mut(),
            );
        }
    }

    pub fn std_add_helpers(&mut self) {}
}

impl Drop for QuickJSRuntime {
    fn drop(&mut self) {
        self.std_free_handlers();
        unsafe {
            JS_FreeRuntime(self.inner);
        }
    }
}

pub struct QuickJS {}

impl QuickJS {
    pub fn null() -> JSValue {
        unsafe { QuickJS_NULL() }
    }

    pub fn undefined() -> JSValue {
        unsafe { QuickJS_UNDEFINED() }
    }

    pub fn r#false() -> JSValue {
        unsafe { QuickJS_FALSE() }
    }

    pub fn r#true() -> JSValue {
        unsafe { QuickJS_TRUE() }
    }

    pub fn exception() -> JSValue {
        unsafe { QuickJS_EXCEPTION() }
    }

    pub fn uninitialized() -> JSValue {
        unsafe { QuickJS_UNINITIALIZED() }
    }

    pub fn to_c_string_len2(
        ctx: *mut JSContext,
        val1: JSValue,
        cesu8: ::std::os::raw::c_int,
    ) -> String {
        unsafe {
            let mut plen: usize = 0;
            let str = JS_ToCStringLen2(ctx, &mut plen, val1, cesu8);
            if str == std::ptr::null() {
                panic!()
            }
            let cstr = CStr::from_ptr(str);
            let string = String::from_utf8_lossy(cstr.to_bytes()).to_string();
            JS_FreeCString(ctx, str);
            string
        }
    }
}
