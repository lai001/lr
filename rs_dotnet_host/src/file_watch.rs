pub type FileChangedFunc = unsafe extern "C" fn(*const std::ffi::c_char);

#[repr(C)]
pub struct FileWatch {
    pub file_changed_func: *const FileChangedFunc,
}

unsafe impl Send for FileWatch {}
unsafe impl Sync for FileWatch {}
