use std::ffi::CString;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[link(name = "kernel32")]
#[link(name = "User32")]
#[cfg(windows)]
extern "stdcall" {
    // pub fn LoadLibraryA(lpFileName: *const u8) -> *const libc::c_void;
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> *mut libc::c_void;
    pub fn GetProcAddress(hModule: *mut libc::c_void, lpProcName: *const u8) -> *mut libc::c_void;
    // pub fn GetLastError() -> i64;
}

#[cfg(windows)]
pub fn to_wstring(str: &str) -> Vec<u16> {
    let v: Vec<u16> = OsStr::new(str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();
    v
}

#[cfg(windows)]
pub fn get_func_ptr(h_module: *mut libc::c_void, name: &str) -> *mut libc::c_void {
    unsafe {
        if let Ok(name) = CString::new(name) {
            let lp_proc_name = name.into_bytes();
            return GetProcAddress(h_module, lp_proc_name.as_ptr());
        }
        return std::ptr::null_mut();
    }
}
