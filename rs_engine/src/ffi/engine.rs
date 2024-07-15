use rs_render::view_mode::EViewModeType;

#[repr(C)]
#[derive(Debug)]
pub struct NativeEngineFunctions {
    set_view_mode: *mut std::ffi::c_void,
    rs_engine_engine_get_camera_mut: *mut std::ffi::c_void,
}

impl NativeEngineFunctions {
    pub fn new() -> NativeEngineFunctions {
        NativeEngineFunctions {
            set_view_mode: rs_engine_Engine_set_view_mode as _,
            rs_engine_engine_get_camera_mut: rs_engine_Engine_get_camera_mut as _,
        }
    }
}

#[repr(C)]
pub struct Engine {
    borrow_mut: *mut crate::engine::Engine,
}

impl Engine {
    pub unsafe fn new(engine: &mut crate::engine::Engine) -> Box<Engine> {
        Box::new(Engine {
            borrow_mut: engine as *mut crate::engine::Engine,
        })
    }
}

#[no_mangle]
pub extern "C" fn rs_engine_Engine_set_view_mode(engine: *mut std::ffi::c_void, mode: i32) {
    let engine: &mut Engine = unsafe { std::mem::transmute(engine) };
    let engine: &mut crate::engine::Engine = unsafe { std::mem::transmute(engine.borrow_mut) };

    let mode = match mode {
        0 => EViewModeType::Wireframe,
        1 => EViewModeType::Lit,
        2 => EViewModeType::Unlit,
        _ => EViewModeType::Lit,
    };
    engine.set_view_mode(mode);
}

#[no_mangle]
pub extern "C" fn rs_engine_Engine_get_camera_mut(
    engine: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    let engine: &mut Engine = unsafe { std::mem::transmute(engine) };
    let engine: &mut crate::engine::Engine = unsafe { std::mem::transmute(engine.borrow_mut) };
    let camera = engine.get_camera_mut();
    Box::into_raw(unsafe { super::camera::Camera::new(camera) }) as _
}
