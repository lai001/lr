pub type Engine = *mut std::ffi::c_void;

pub trait Plugin {
    fn tick(&mut self, engine: Engine);
}

#[link(name = "rs_engine.dll")]
extern "C" {
    pub fn rs_engine_Engine_set_view_mode(engine: Engine, mode: i32);
}

pub mod symbol_name {
    pub const CREATE_PLUGIN: &str = "create_plugin";
}

pub mod signature {
    use super::Plugin;

    pub type CreatePlugin = fn() -> Box<dyn Plugin>;
}
