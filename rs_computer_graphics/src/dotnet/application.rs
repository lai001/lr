use crate::ffi::{
    native_keyboard_input::NativeKeyboardInput, native_queue::NativeWGPUQueue,
    native_texture_view::NativeWGPUTextureView,
};
use std::sync::Mutex;

pub type RuntimeInstanceType = *mut libc::c_void;

pub type ApplicationRedrawRequested = unsafe extern "C" fn(
    appPtr: RuntimeInstanceType,
    native_texture_view: NativeWGPUTextureView,
    native_queue: NativeWGPUQueue,
);

pub type ApplicationKeyboardInput =
    unsafe extern "C" fn(appPtr: RuntimeInstanceType, keyboard_input: NativeKeyboardInput);

pub type ApplicationCursorMoved =
    unsafe extern "C" fn(appPtr: RuntimeInstanceType, position: winit::dpi::PhysicalPosition<f64>);

#[repr(C)]
#[derive(Debug)]
pub struct RuntimeApplicationFunctions {
    application_redraw_requested: *mut ApplicationRedrawRequested,
    application_keyboard_input: *mut ApplicationKeyboardInput,
    application_cursor_moved: *mut ApplicationCursorMoved,
}

unsafe impl Send for RuntimeApplicationFunctions {}
unsafe impl Sync for RuntimeApplicationFunctions {}

lazy_static! {
    pub static ref GLOBAL_RUNTIME_APPLICATION_FUNCTIONS: Mutex<RuntimeApplicationFunctions> =
        Mutex::new(RuntimeApplicationFunctions {
            application_redraw_requested: std::ptr::null_mut(),
            application_keyboard_input: std::ptr::null_mut(),
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

    pub fn redraw_requested(
        &mut self,
        native_texture_view: NativeWGPUTextureView,
        native_queue: NativeWGPUQueue,
    ) {
        unsafe {
            let func_ptr: ApplicationRedrawRequested = std::mem::transmute(
                GLOBAL_RUNTIME_APPLICATION_FUNCTIONS
                    .lock()
                    .unwrap()
                    .application_redraw_requested,
            );
            func_ptr(self.instance, native_texture_view, native_queue);
        }
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

    pub fn cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
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
}
