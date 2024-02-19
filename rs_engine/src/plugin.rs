pub trait Plugin {
    fn tick(&mut self);
}

pub mod symbol_name {
    pub const FROM: &str = "from";
}

pub mod signature {
    use super::Plugin;
    use crate::plugin_context::PluginContext;
    use std::sync::{Arc, Mutex};

    pub type From = fn(plugin_context: Arc<Mutex<PluginContext>>) -> Box<dyn Plugin>;
}
