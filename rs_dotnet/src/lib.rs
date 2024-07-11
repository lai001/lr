pub mod dotnet;
#[cfg(windows)]
mod windows;
#[macro_use]
#[cfg(windows)]
extern crate lazy_static;
