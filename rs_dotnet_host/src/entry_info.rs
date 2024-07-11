use super::application::{RuntimeApplicationFunctions, RuntimeInstanceType};
use crate::file_watch::FileWatch;
use rs_engine::ffi::NativeEngineFunctions;
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
#[derive(Debug)]
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
}
