#[cfg(not(target_os = "android"))]
mod windows_logger;
#[cfg(not(target_os = "android"))]
pub use windows_logger::*;
#[cfg(target_os = "android")]
mod android_logger;
#[cfg(target_os = "android")]
pub use android_logger::*;

bitflags::bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct SlotFlags: u8 {
        const Level = 1;
        const ThreadName = 1 << 1;
        const FileLine = 1 << 2;
        const Timestamp = 1 << 3;
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoggerConfiguration {
    pub is_write_to_file: bool,
    pub is_flush_before_drop: bool,
    pub slot_flags: SlotFlags,
}
