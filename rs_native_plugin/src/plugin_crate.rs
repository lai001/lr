use rs_engine::engine::Engine;

pub trait Plugin {
    fn tick(&mut self, engine: &mut Engine);
}

pub mod symbol_name {
    pub const CREATE_PLUGIN: &str = "create_plugin";
}

pub mod signature {
    use super::Plugin;

    pub type CreatePlugin = fn() -> Box<dyn Plugin>;
}
