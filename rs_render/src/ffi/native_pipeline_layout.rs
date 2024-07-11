use wgpu::PipelineLayout;

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUPipelineLayoutFunctions {
    pub native_pipeline_layout_delete: *mut std::ffi::c_void,
}

impl NativeWGPUPipelineLayoutFunctions {
    pub fn new() -> NativeWGPUPipelineLayoutFunctions {
        NativeWGPUPipelineLayoutFunctions {
            native_pipeline_layout_delete: nativePipelineLayoutDelete as *mut std::ffi::c_void,
        }
    }
}

#[no_mangle]
pub extern "C" fn nativePipelineLayoutDelete(native_object: *mut PipelineLayout) {
    if !native_object.is_null() {
        log::trace!(
            "nativePipelineLayoutDelete(native_object: {:?})",
            native_object
        );
        unsafe {
            let _ = Box::from_raw(native_object);
        };
    }
}
