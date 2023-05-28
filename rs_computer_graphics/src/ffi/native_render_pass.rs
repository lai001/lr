use wgpu::{RenderPass, RenderPipeline};

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPURenderPassFunctions {
    pub native_render_pass_set_pipeline: *mut libc::c_void,
    pub native_render_pass_draw: *mut libc::c_void,
    pub native_render_pass_delete: *mut libc::c_void,
}

impl NativeWGPURenderPassFunctions {
    pub fn new() -> NativeWGPURenderPassFunctions {
        NativeWGPURenderPassFunctions {
            native_render_pass_set_pipeline: nativeRenderPassSetPipeline as *mut libc::c_void,
            native_render_pass_draw: nativeRenderPassDraw as *mut libc::c_void,
            native_render_pass_delete: nativeRenderPassDelete as *mut libc::c_void,
        }
    }
}

#[no_mangle]
pub extern "C" fn nativeRenderPassSetPipeline(
    render_pass: *mut RenderPass,
    render_pipeline: *mut RenderPipeline,
) {
    assert_ne!(render_pass, std::ptr::null_mut());
    assert_ne!(render_pipeline, std::ptr::null_mut());
    unsafe {
        (*render_pass).set_pipeline(&(*render_pipeline));
        log::trace!(
            "nativeRenderPassSetPipeline(render_pass: {:?}, render_pipeline: {:?})",
            render_pass,
            render_pipeline
        );
    }
}

#[no_mangle]
pub extern "C" fn nativeRenderPassDraw(
    render_pass: *mut RenderPass,
    vertices: crate::util::Range<u32>,
    instances: crate::util::Range<u32>,
) {
    assert_ne!(render_pass, std::ptr::null_mut());
    unsafe {
        log::trace!(
            "nativeRenderPassDraw(render_pass: {:?}, vertices: {:?}, instances: {:?})",
            render_pass,
            vertices.clone(),
            instances.clone()
        );
        (*render_pass).draw(vertices.to_std_range(), instances.to_std_range());
    }
}

#[no_mangle]
pub extern "C" fn nativeRenderPassDelete(native_object: *mut wgpu::RenderPass) {
    if !native_object.is_null() {
        log::trace!("nativeRenderPassDelete(native_object: {:?})", native_object);
        unsafe { Box::from_raw(native_object) };
    }
}
