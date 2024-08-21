pub type Engine = *mut std::ffi::c_void;

pub trait Plugin {
    fn tick(&mut self, engine: Engine);
}

#[link(name = "rs_engine.dll")]
extern "C" {
    pub fn rs_engine_Engine_set_view_mode(engine: Engine, mode: i32);
}
