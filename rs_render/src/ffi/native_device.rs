use crate::ffi::gpu_texture_format::NativeWGPUTextureFormat;
use rs_foundation::ffi_to_rs_string;
use std::ffi;
use wgpu::{CommandEncoder, Device, PipelineLayout, RenderPipeline, ShaderModule};

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUDevice {
    pub device: *mut Device,
}

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUDeviceFunctions {
    pub native_create_shader_module: *mut std::ffi::c_void,
    pub native_create_pipeline_layout: *mut std::ffi::c_void,
    pub native_create_render_pipeline: *mut std::ffi::c_void,
    pub native_create_command_encoder: *mut std::ffi::c_void,
}

impl NativeWGPUDeviceFunctions {
    pub fn new() -> NativeWGPUDeviceFunctions {
        NativeWGPUDeviceFunctions {
            native_create_shader_module: nativeDeviceCreateShaderModule as *mut std::ffi::c_void,
            native_create_pipeline_layout: nativeDeviceCreatePipelineLayout
                as *mut std::ffi::c_void,
            native_create_render_pipeline: nativeDeviceCreateRenderPipeline
                as *mut std::ffi::c_void,
            native_create_command_encoder: nativeDeviceCreateCommandEncoder
                as *mut std::ffi::c_void,
        }
    }
}

#[no_mangle]
pub extern "C" fn nativeDeviceCreateShaderModule(
    device: *mut Device,
    label: *const ffi::c_char,
    path: *const ffi::c_char,
) -> *mut ShaderModule {
    assert_ne!(device, std::ptr::null_mut());
    assert_ne!(path, std::ptr::null());
    unsafe {
        let path = ffi_to_rs_string(path).unwrap();
        let code = std::fs::read_to_string(path.clone()).unwrap();
        let shader_code = wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&code));
        let shader = (*device).create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&ffi_to_rs_string(label).unwrap_or(String::new())),
            source: shader_code,
        });
        let handle = Box::into_raw(Box::new(shader));
        log::trace!(
            "nativeDeviceCreateShaderModule(device: {:?}, label: {:?}, path: {:?}) -> {:?}",
            device,
            Some(&ffi_to_rs_string(label).unwrap_or(String::new())),
            path,
            handle
        );
        handle
    }
}

#[no_mangle]
pub extern "C" fn nativeDeviceCreatePipelineLayout(
    device: *mut Device,
    label: *const ffi::c_char,
) -> *mut PipelineLayout {
    assert_ne!(device, std::ptr::null_mut());
    unsafe {
        let pipeline_layout = (*device).create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&ffi_to_rs_string(label).unwrap_or(String::new())),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let handle = Box::into_raw(Box::new(pipeline_layout));
        log::trace!(
            "nativeDeviceCreatePipelineLayout(device: {:?}, label: {:?}) -> {:?}",
            device,
            Some(&ffi_to_rs_string(label).unwrap_or(String::new())),
            handle
        );
        handle
    }
}

#[no_mangle]
pub extern "C" fn nativeDeviceCreateRenderPipeline(
    device: *mut Device,
    label: *const ffi::c_char,
    pipeline_layout: *mut PipelineLayout,
    shader: *mut ShaderModule,
    swapchain_format: NativeWGPUTextureFormat,
) -> *mut RenderPipeline {
    assert_ne!(device, std::ptr::null_mut());
    assert_ne!(pipeline_layout, std::ptr::null_mut());
    assert_ne!(shader, std::ptr::null_mut());
    unsafe {
        let render_pipeline = (*device).create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&ffi_to_rs_string(label).unwrap_or(String::new())),
            layout: Some(&(*pipeline_layout)),
            vertex: wgpu::VertexState {
                module: &(*shader),
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &(*shader),
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.to_texture_format().into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let handle = Box::into_raw(Box::new(render_pipeline));
        log::trace!(
            "nativeDeviceCreateRenderPipeline(device: {:?}, label: {:?}, pipeline_layout: {:?}, shader: {:?}, swapchain_format: {:?}) -> {:?}",
            device,
            Some(&ffi_to_rs_string(label).unwrap_or(String::new())),
            pipeline_layout,
            shader,
            swapchain_format,
            handle,
        );
        handle
    }
}

#[no_mangle]
pub extern "C" fn nativeDeviceCreateCommandEncoder(device: *mut Device) -> *mut CommandEncoder {
    assert_ne!(device, std::ptr::null_mut());
    unsafe {
        let encoder =
            (*device).create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let handle = Box::into_raw(Box::new(encoder));
        log::trace!(
            "nativeDeviceCreateCommandEncoder(device: {:?}) -> {:?}",
            device,
            handle
        );
        handle
    }
}
