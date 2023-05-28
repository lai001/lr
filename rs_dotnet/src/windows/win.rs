use std::ffi::CString;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[link(name = "kernel32")]
#[link(name = "User32")]
#[cfg(windows)]
extern "stdcall" {
    pub fn LoadLibraryA(lpFileName: *const u8) -> *const libc::c_void;
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> *mut libc::c_void;
    pub fn GetProcAddress(hModule: *mut libc::c_void, lpProcName: *const u8) -> *mut libc::c_void;
    pub fn GetLastError() -> i64;
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
pub fn get_func_ptr(h: *mut libc::c_void, name: &str) -> *mut libc::c_void {
    unsafe {
        let s = CString::new(name).unwrap().into_bytes();
        GetProcAddress(h, s.as_ptr())
    }
}
