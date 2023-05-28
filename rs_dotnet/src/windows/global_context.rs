use std::sync::Mutex;

use crate::dotnet::{
    HostfxrCloseFn, HostfxrGetRuntimeDelegateFn, HostfxrInitializeForRuntimeConfigFn,
};

pub struct Context {
    pub initialize_for_runtime_config_func_ptr: *mut HostfxrInitializeForRuntimeConfigFn,
    pub get_runtime_delegate_func_ptr: *mut HostfxrGetRuntimeDelegateFn,
    pub close_func_ptr: *mut HostfxrCloseFn,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

lazy_static! {
    pub static ref GLOBAL_CONTEXT: Mutex<Context> = Mutex::new(Context {
        initialize_for_runtime_config_func_ptr: std::ptr::null_mut(),
        get_runtime_delegate_func_ptr: std::ptr::null_mut(),
        close_func_ptr: std::ptr::null_mut(),
    });
}
