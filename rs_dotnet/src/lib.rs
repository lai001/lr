pub mod dotnet;
#[cfg(windows)]
pub mod windows;
#[macro_use]
#[cfg(windows)]
extern crate lazy_static;
