#[cfg(feature = "plugin_shared_lib")]
pub mod plugin;
#[cfg(feature = "plugin_shared_lib")]
pub use crate::plugin::*;

#[cfg(feature = "plugin_shared_crate")]
pub mod plugin_crate;
#[cfg(feature = "plugin_shared_crate")]
pub use crate::plugin_crate::*;
