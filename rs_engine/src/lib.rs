pub mod camera;
#[cfg(not(target_os = "android"))]
pub mod camera_input_event_handle;
pub mod engine;
pub mod error;
pub mod file_type;
pub mod handle;
pub mod logger;
pub mod primitive_data;
pub mod resource_manager;
pub mod rotator;
pub mod sync;
pub mod thread_pool;

pub const ASSET_SCHEME: &str = "asset";

pub fn build_asset_url(name: &str) -> Result<url::Url, url::ParseError> {
    url::Url::parse(&format!("{}://{}", ASSET_SCHEME, name))
}
