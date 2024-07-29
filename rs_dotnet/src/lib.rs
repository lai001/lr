pub mod dotnet;
pub mod error;
#[cfg(windows)]
mod windows;
#[macro_use]
#[cfg(windows)]
extern crate lazy_static;
