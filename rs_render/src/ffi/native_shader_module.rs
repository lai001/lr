#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUShaderModule {
    pub shader_module: *mut wgpu::ShaderModule,
}

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUShaderModuleFunctions {
    pub native_shader_module_delete: *mut std::ffi::c_void,
}

impl NativeWGPUShaderModuleFunctions {
    pub fn new() -> NativeWGPUShaderModuleFunctions {
        NativeWGPUShaderModuleFunctions {
            native_shader_module_delete: nativeShaderModuleDelete as *mut std::ffi::c_void,
        }
    }
}

#[no_mangle]
pub extern "C" fn nativeShaderModuleDelete(native_object: *mut wgpu::ShaderModule) {
    if !native_object.is_null() {
        unsafe {
            let _ = Box::from_raw(native_object);
        };
        log::trace!(
            "nativeShaderModuleDelete(native_object: {:?})",
            native_object
        );
    }
}
