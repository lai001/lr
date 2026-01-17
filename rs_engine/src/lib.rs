pub mod actor;
pub mod camera;
pub mod camera_component;
pub mod camera_input_event_handle;
pub mod cluster_light;
pub mod collision_componenet;
pub mod components;
pub mod console_cmd;
pub mod content;
pub mod debug_show_flag;
pub mod default_textures;
pub mod directional_light;
pub mod drawable;
pub mod engine;
pub mod error;
pub mod ffi;
pub mod frame_sync;
pub mod handle;
pub mod input_mode;
pub mod input_type;
pub mod keys_detector;
pub mod kinematic_component;
pub mod logger;
pub mod mesh_buffer;
pub mod mipmap_generator;
pub mod misc;
#[cfg(feature = "network")]
pub mod network;
pub mod particle;
pub mod physics_debug_render;
pub mod planar_reflection;
pub mod player_viewport;
#[cfg(feature = "plugin_shared_crate")]
pub mod plugin;
pub mod property;
pub mod render_thread_mode;
pub mod resource_manager;
pub mod rotator;
pub mod scene_node;
pub mod skeleton_animation_provider;
pub mod skeleton_mesh_component;
pub mod standalone;
pub mod static_mesh_component;
pub mod static_virtual_texture_source;
pub mod sync;
pub mod uniform_map;
pub mod url_extension;

pub use rs_core_minimal::file_type;
pub use rs_core_minimal::thread_pool;

pub const ASSET_SCHEME: &str = "asset";
pub const CONTENT_SCHEME: &str = "content";
pub const DERIVE_DATA_SCHEME: &str = "derivedata";
pub const ASSET_ROOT: &str = "asset";
pub const CONTENT_ROOT: &str = "Content";
pub const BUILT_IN_RESOURCE: &str = "builtinresouce";

pub fn build_asset_url(name: impl AsRef<str>) -> Result<url::Url, url::ParseError> {
    url::Url::parse(&format!("{ASSET_SCHEME}://{ASSET_ROOT}/{}", name.as_ref()))
}

pub fn build_content_file_url(name: impl AsRef<str>) -> Result<url::Url, url::ParseError> {
    url::Url::parse(&format!(
        "{CONTENT_SCHEME}://{CONTENT_ROOT}/{}",
        name.as_ref()
    ))
}

pub fn build_built_in_resouce_url(name: impl AsRef<str>) -> Result<url::Url, url::ParseError> {
    url::Url::parse(&format!("{}://{}", BUILT_IN_RESOURCE, name.as_ref()))
}

pub fn build_derive_data_url(name: impl AsRef<str>) -> Result<url::Url, url::ParseError> {
    url::Url::parse(&format!("{}://{}", DERIVE_DATA_SCHEME, name.as_ref()))
}

#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 100);
