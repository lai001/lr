#[repr(C)]
pub struct NativeCameraFunctions {
    set_window_size: *mut std::ffi::c_void,
    drop: *mut std::ffi::c_void,
}

impl NativeCameraFunctions {
    pub fn new() -> NativeCameraFunctions {
        NativeCameraFunctions {
            set_window_size: rs_engine_Camera_set_window_size as _,
            drop: rs_engine_Camera_drop as _,
        }
    }
}

#[repr(C)]
pub struct Camera {
    borrow_mut: *mut crate::camera::Camera,
}

impl Camera {
    pub unsafe fn new(camera: &mut crate::camera::Camera) -> Box<Camera> {
        Box::new(Camera {
            borrow_mut: camera as *mut crate::camera::Camera,
        })
    }
}

#[no_mangle]
pub extern "C" fn rs_engine_Camera_set_window_size(
    camera: *mut std::ffi::c_void,
    window_width: u32,
    window_height: u32,
) {
    let camera: &mut Camera = unsafe { std::mem::transmute(camera) };
    let camera: &mut crate::camera::Camera = unsafe { std::mem::transmute(camera.borrow_mut) };
    camera.set_window_size(window_width, window_height);
}

#[no_mangle]
pub extern "C" fn rs_engine_Camera_drop(camera: *mut std::ffi::c_void) {
    unsafe {
        let _ = Box::from_raw(camera);
    };
}
