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
