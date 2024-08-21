#[cfg(feature = "plugin_shared_lib")]
pub mod plugin;
#[cfg(feature = "plugin_shared_lib")]
pub use crate::plugin::*;

#[cfg(any(
    feature = "plugin_shared_crate_export",
    feature = "plugin_shared_crate_import"
))]
pub mod plugin_crate;
#[cfg(any(
    feature = "plugin_shared_crate_export",
    feature = "plugin_shared_crate_import"
))]
pub use crate::plugin_crate::*;

pub mod symbol_name {
    pub const CREATE_PLUGIN: &str = "create_plugin";
}

pub mod signature {
    use super::Plugin;

    pub type CreatePlugin = fn() -> Box<dyn Plugin>;
}
