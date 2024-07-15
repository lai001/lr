use super::application::{RuntimeApplicationFunctions, RuntimeInstanceType};
use crate::{application::GLOBAL_RUNTIME_APPLICATION_FUNCTIONS, file_watch::FileWatch};
use rs_engine::ffi::{camera::NativeCameraFunctions, engine::NativeEngineFunctions};
use rs_render::ffi::{
    native_command_encoder::NativeWGPUCommandEncoderFunctions,
    native_device::NativeWGPUDeviceFunctions,
    native_pipeline_layout::NativeWGPUPipelineLayoutFunctions,
    native_queue::NativeWGPUQueueFunctions, native_render_pass::NativeWGPURenderPassFunctions,
    native_render_pipeline::NativeWGPURenderPipelineFunctions,
    native_shader_module::NativeWGPUShaderModuleFunctions,
};
// use wgpu::Device;

#[repr(C)]
pub struct EntryInfo {
    pub runtime_application: RuntimeInstanceType,
    pub runtime_application_functions: *mut RuntimeApplicationFunctions,
    pub runtime_file_watch: *mut FileWatch,
    pub native_device_functions: NativeWGPUDeviceFunctions,
    // pub native_device: *mut Device,
    pub native_command_encoder_functions: NativeWGPUCommandEncoderFunctions,
    pub native_render_pass_functions: NativeWGPURenderPassFunctions,
    pub native_queue_functions: NativeWGPUQueueFunctions,
    pub native_shader_module_functions: NativeWGPUShaderModuleFunctions,
    pub native_render_pipeline_functions: NativeWGPURenderPipelineFunctions,
    pub native_pipeline_layout_functions: NativeWGPUPipelineLayoutFunctions,
    pub native_engine_functions: NativeEngineFunctions,
    pub native_camera_functions: NativeCameraFunctions,
}

impl EntryInfo {
    pub fn new(file_watch: &mut FileWatch) -> EntryInfo {
        EntryInfo {
            runtime_application: std::ptr::null_mut(),
            runtime_application_functions: (&mut GLOBAL_RUNTIME_APPLICATION_FUNCTIONS
                .lock()
                .unwrap())
                as &mut RuntimeApplicationFunctions
                as *mut _
                as *mut RuntimeApplicationFunctions,
            runtime_file_watch: file_watch as *mut _ as *mut FileWatch,
            native_device_functions: NativeWGPUDeviceFunctions::new(),
            // native_device: device as *mut _ as *mut wgpu::Device,
            native_command_encoder_functions: NativeWGPUCommandEncoderFunctions::new(),
            native_render_pass_functions: NativeWGPURenderPassFunctions::new(),
            native_queue_functions: NativeWGPUQueueFunctions::new(),
            native_shader_module_functions: NativeWGPUShaderModuleFunctions::new(),
            native_render_pipeline_functions: NativeWGPURenderPipelineFunctions::new(),
            native_pipeline_layout_functions: NativeWGPUPipelineLayoutFunctions::new(),
            native_engine_functions: NativeEngineFunctions::new(),
            native_camera_functions: NativeCameraFunctions::new(),
        }
    }
}
