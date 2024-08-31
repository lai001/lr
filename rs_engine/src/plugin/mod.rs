pub mod plugin_crate;

pub mod symbol_name {
    pub const CREATE_PLUGIN: &str = "create_plugin";
}

pub mod signature {
    use super::plugin_crate::Plugin;
    pub type CreatePlugin = fn() -> Box<dyn Plugin>;
}
