use wgpu::{CommandBuffer, Queue};

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUQueue {
    pub queue: *const Queue,
}

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUQueueFunctions {
    pub native_queue_submit: *mut std::ffi::c_void,
}

impl NativeWGPUQueueFunctions {
    pub fn new() -> NativeWGPUQueueFunctions {
        NativeWGPUQueueFunctions {
            native_queue_submit: nativeQueueSubmit as *mut std::ffi::c_void,
        }
    }
}

#[no_mangle]
pub extern "C" fn nativeQueueSubmit(queue: *mut Queue, command_buffer: *mut CommandBuffer) {
    assert_ne!(queue, std::ptr::null_mut());
    assert_ne!(command_buffer, std::ptr::null_mut());
    unsafe {
        (*queue).submit(std::iter::once(*Box::from_raw(command_buffer)));
        // Box::into_raw(Box::new(command_buffer));
        log::trace!(
            "nativeQueueSubmit(queue: {:?}, command_buffer: {:?})",
            queue,
            command_buffer
        );
    }
}
