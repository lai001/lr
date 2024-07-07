use rs_render::view_mode::EViewModeType;

pub struct Engine {
    inner: *mut crate::engine::Engine,
}

impl Engine {
    pub unsafe fn new(engine: &mut crate::engine::Engine) -> Box<Engine> {
        Box::new(Engine {
            inner: engine as *mut crate::engine::Engine,
        })
    }
}

#[no_mangle]
pub extern "C" fn rs_engine_Engine_set_view_mode(engine: *mut std::ffi::c_void, mode: i32) {
    let engine: &mut Engine = unsafe { std::mem::transmute(engine) };
    let engine: &mut crate::engine::Engine = unsafe { std::mem::transmute(engine.inner) };

    let mode = match mode {
        0 => EViewModeType::Wireframe,
        1 => EViewModeType::Lit,
        2 => EViewModeType::Unlit,
        _ => EViewModeType::Lit,
    };
    engine.set_view_mode(mode);
}
