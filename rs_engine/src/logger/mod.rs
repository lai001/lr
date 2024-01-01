#[cfg(not(target_os = "android"))]
mod windows_logger;
#[cfg(not(target_os = "android"))]
pub use windows_logger::*;
#[cfg(target_os = "android")]
mod android_logger;
#[cfg(target_os = "android")]
pub use android_logger::*;
