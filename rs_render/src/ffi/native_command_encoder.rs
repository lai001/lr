use wgpu::{CommandBuffer, CommandEncoder};

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUCommandEncoderFunctions {
    pub begin_render_pass: *mut std::ffi::c_void,
    pub finish: *mut std::ffi::c_void,
}

impl NativeWGPUCommandEncoderFunctions {
    pub fn new() -> NativeWGPUCommandEncoderFunctions {
        NativeWGPUCommandEncoderFunctions {
            begin_render_pass: nativeCommandEncoderBeginRenderPass as *mut std::ffi::c_void,
            finish: nativeCommandEncoderFinish as *mut std::ffi::c_void,
        }
    }
}

#[no_mangle]
pub extern "C" fn nativeCommandEncoderBeginRenderPass(
    encoder: *mut CommandEncoder,
    output_view: *mut wgpu::TextureView,
) -> *mut wgpu::RenderPass<'static> {
    assert_ne!(encoder, std::ptr::null_mut());
    unsafe {
        let render_pass = (*encoder).begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &(*output_view),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let handle = Box::into_raw(Box::new(render_pass));
        log::trace!(
            "nativeCommandEncoderBeginRenderPass(encoder: {:?}, output_view: {:?}) -> {:?}",
            encoder,
            output_view,
            handle
        );
        handle
    }
}

#[no_mangle]
pub extern "C" fn nativeCommandEncoderFinish(
    command_encoder: *mut CommandEncoder,
) -> *mut CommandBuffer {
    assert_ne!(command_encoder, std::ptr::null_mut());
    unsafe {
        let command_buffer = Box::from_raw(command_encoder).finish();
        let handle = Box::into_raw(Box::new(command_buffer));
        log::trace!(
            "nativeCommandEncoderFinish(command_encoder: {:?}) -> {:?}",
            command_encoder,
            handle
        );
        // Box::into_raw(Box::new(command_encoder));
        handle
    }
}
