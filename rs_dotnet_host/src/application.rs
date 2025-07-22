use crate::native_keyboard_input::NativeKeyboardInput;
use lazy_static::lazy_static;
use rs_render::ffi::{native_queue::NativeWGPUQueue, native_texture_view::NativeWGPUTextureView};
use std::sync::Mutex;

pub type RuntimeInstanceType = *mut std::ffi::c_void;

pub type ApplicationRedrawRequested = unsafe extern "C" fn(
    app_ptr: RuntimeInstanceType,
    native_texture_view: NativeWGPUTextureView,
    native_queue: NativeWGPUQueue,
);

pub type ApplicationKeyboardInput =
    unsafe extern "C" fn(app_ptr: RuntimeInstanceType, keyboard_input: NativeKeyboardInput);

pub type ApplicationCursorMoved =
    unsafe extern "C" fn(app_ptr: RuntimeInstanceType, position: glam::DVec2);

pub type ApplicationTick =
    unsafe extern "C" fn(app_ptr: RuntimeInstanceType, engine: *mut rs_engine::ffi::engine::Engine);

#[repr(C)]
#[derive(Debug)]
pub struct RuntimeApplicationFunctions {
    application_keyboard_input: *mut ApplicationKeyboardInput,
    application_cursor_moved: *mut ApplicationCursorMoved,
    application_tick: *mut ApplicationTick,
}

unsafe impl Send for RuntimeApplicationFunctions {}
unsafe impl Sync for RuntimeApplicationFunctions {}

lazy_static! {
    // TODO: Optimization, remove mutex
    pub static ref GLOBAL_RUNTIME_APPLICATION_FUNCTIONS: Mutex<RuntimeApplicationFunctions> =
        Mutex::new(RuntimeApplicationFunctions {
            application_keyboard_input: std::ptr::null_mut(),
            application_tick: std::ptr::null_mut(),
            application_cursor_moved: std::ptr::null_mut(),
        });
}

#[repr(C)]
#[derive(Debug)]
pub struct RuntimeApplication {
    instance: RuntimeInstanceType,
}

impl RuntimeApplication {
    pub fn new(instance: RuntimeInstanceType) -> RuntimeApplication {
        RuntimeApplication { instance }
    }

    pub fn keyboard_input(&mut self, keyboard_input: NativeKeyboardInput) {
        unsafe {
            let func_ptr: ApplicationKeyboardInput = std::mem::transmute(
                GLOBAL_RUNTIME_APPLICATION_FUNCTIONS
                    .lock()
                    .unwrap()
                    .application_keyboard_input,
            );
            func_ptr(self.instance, keyboard_input);
        }
    }

    pub fn cursor_moved(&mut self, position: glam::DVec2) {
        unsafe {
            let func_ptr: ApplicationCursorMoved = std::mem::transmute(
                GLOBAL_RUNTIME_APPLICATION_FUNCTIONS
                    .lock()
                    .unwrap()
                    .application_cursor_moved,
            );
            func_ptr(self.instance, position);
        }
    }

    pub fn tick(&mut self, engine: &mut rs_engine::engine::Engine) {
        unsafe {
            let application_tick = GLOBAL_RUNTIME_APPLICATION_FUNCTIONS
                .lock()
                .unwrap()
                .application_tick;
            if application_tick.is_null() {
                return;
            }
            let func_ptr: ApplicationTick = std::mem::transmute(application_tick);
            let ffi_engine = rs_engine::ffi::engine::Engine::new(engine);
            let raw = Box::into_raw(ffi_engine);
            func_ptr(self.instance, raw);
            let _ = Box::from_raw(raw);
        }
    }
}
