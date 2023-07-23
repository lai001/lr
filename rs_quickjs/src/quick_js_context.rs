use crate::{quick_js_runtime::QuickJSRuntime, quickjs_bindings::*};
use std::{cell::RefCell, ffi::CString, sync::Arc};

pub struct QuickJSContext {
    inner: *mut JSContext,
    runtime: Arc<RefCell<QuickJSRuntime>>,
}

impl QuickJSContext {
    pub fn new(runtime: Arc<RefCell<QuickJSRuntime>>) -> QuickJSContext {
        unsafe {
            QuickJSContext {
                inner: JS_NewContext(runtime.borrow_mut().inner),
                runtime: runtime.clone(),
            }
        }
    }

    pub fn std_add_helpers(&self) {
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

    pub fn init_module_std(&self) {
        unsafe {
            js_init_module_std(self.inner, "std".as_ptr() as *const i8);
        }
    }

    pub fn init_module_os(&self) {
        unsafe {
            js_init_module_os(self.inner, "os".as_ptr() as *const i8);
        }
    }

    pub fn eval_file_module(&self, filename: &str) {
        unsafe {
            let c_str = CString::new(filename).unwrap();
            QuickjsHelper_evalFile(
                self.inner,
                c_str.as_ptr() as *const i8,
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

    pub fn free_value(&self, value: JSValue) {
        unsafe {
            QuickJS_FreeValue(self.inner, value);
        }
    }

    pub fn set_property_str(&self, this_obj: JSValue, prop: &str, val: JSValue) {
        let c_str = CString::new(prop).unwrap();
        unsafe {
            JS_SetPropertyStr(self.inner, this_obj, c_str.as_ptr(), val);
        }
    }

    pub fn new_c_function(&self, func: JSCFunction, name: &str, length: i32) -> JSValue {
        let c_str = CString::new(name).unwrap();
        unsafe { QuickJS_NewCFunction(self.inner, func, c_str.as_ptr() as *const i8, length) }
    }

    pub fn new_int64(&self, value: i64) -> JSValue {
        unsafe { QuickJS_NewInt64(self.inner, value) }
    }

    pub fn get_property_str(
        &mut self,
        this_obj: JSValue,
        name: &str,
        mut closure: impl FnMut(&mut QuickJSContext, JSValue) -> (),
    ) {
        let c_str = CString::new(name).unwrap();
        unsafe {
            let object = JS_GetPropertyStr(self.inner, this_obj, c_str.as_ptr());
            closure(self, object);
            self.free_value(object)
        }
    }

    pub fn new_atom(&self, name: &str) -> JSAtom {
        let c = CString::new(name).unwrap();
        unsafe {
            let atom = JS_NewAtom(self.inner, c.as_ptr());
            atom
        }
    }

    pub fn new_atom_uint32(&self, n: u32) -> JSAtom {
        unsafe {
            let atom = JS_NewAtomUInt32(self.inner, n);
            atom
        }
    }

    pub fn free_atom(&self, atom: JSAtom) {
        unsafe {
            JS_FreeAtom(self.inner, atom);
        }
    }

    pub fn has_property(&self, this_obj: JSValue, prop: JSAtom) -> bool {
        unsafe { JS_HasProperty(self.inner, this_obj, prop) != 0 }
    }

    pub fn get_runtime(&self) -> Arc<RefCell<QuickJSRuntime>> {
        self.runtime.clone()
    }

    pub fn new_object(&self) -> JSValue {
        unsafe { JS_NewObject(self.inner) }
    }

    pub fn set_property_function_list(&self, obj: JSValue, tab: &[JSCFunctionListEntry]) {
        unsafe {
            if tab.is_empty() {
                JS_SetPropertyFunctionList(self.inner, obj, std::ptr::null(), 0);
            } else {
                JS_SetPropertyFunctionList(
                    self.inner,
                    obj,
                    tab.as_ptr(),
                    tab.len() as ::std::os::raw::c_int,
                );
            }
        }
    }

    pub fn new_c_function2(
        &self,
        func: JSCFunction,
        name: &str,
        length: ::std::os::raw::c_int,
        cproto: EJSCFunctionType,
        magic: ::std::os::raw::c_int,
    ) -> JSValue {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let cfunc = JS_NewCFunction2(
                self.inner,
                func,
                c_str.as_ptr(),
                length,
                *(&cproto) as i32,
                magic,
            );
            return cfunc;
        }
    }

    pub fn new_constructor_function(&self, func: JSCFunction, name: &str) -> JSValue {
        self.new_c_function2(func, name, 0, EJSCFunctionType::Constructor, 0)
    }

    pub fn set_constructor(&self, func_obj: JSValue, proto: JSValue) {
        unsafe {
            JS_SetConstructor(self.inner, func_obj, proto);
        }
    }
    pub fn set_class_proto(&self, class_id: JSClassID, obj: JSValue) {
        unsafe {
            JS_SetClassProto(self.inner, class_id, obj);
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
