pub trait Plugin {
    fn tick(&mut self);
}

pub mod symbol_name {
    pub const CREATE_PLUGIN: &str = "create_plugin";
}

pub mod signature {
    use super::Plugin;
    use crate::plugin_context::PluginContext;
    use std::sync::{Arc, Mutex};

    pub type CreatePlugin = fn(plugin_context: Arc<Mutex<PluginContext>>) -> Box<dyn Plugin>;
}
