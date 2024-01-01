pub struct NativeWindow {
    a_native_window: *mut ndk_sys::ANativeWindow,
}

impl NativeWindow {
    pub fn new(env: &mut jni::JNIEnv, surface: jni::sys::jobject) -> Option<Self> {
        let a_native_window =
            unsafe { ndk_sys::ANativeWindow_fromSurface(env.get_raw(), surface as *mut _) };
        if a_native_window.is_null() {
            None
        } else {
            Some(Self { a_native_window })
        }
    }

    pub fn get_width(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getWidth(self.a_native_window) as u32 }
    }

    pub fn get_height(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getHeight(self.a_native_window) as u32 }
    }

    pub fn get_format(&self) -> i32 {
        unsafe { ndk_sys::ANativeWindow_getFormat(self.a_native_window) }
    }

    pub fn set_buffers_geometry(&self, width: u32, height: u32, format: i32) -> i32 {
        unsafe {
            ndk_sys::ANativeWindow_setBuffersGeometry(
                self.a_native_window,
                width as i32,
                height as i32,
                format,
            )
        }
    }
}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::ANativeWindow_release(self.a_native_window);
        }
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for NativeWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        let mut handle = raw_window_handle::AndroidNdkWindowHandle::empty();
        handle.a_native_window = self.a_native_window as *mut _ as *mut core::ffi::c_void;
        raw_window_handle::RawWindowHandle::AndroidNdk(handle)
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for NativeWindow {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        raw_window_handle::RawDisplayHandle::Android(
            raw_window_handle::AndroidDisplayHandle::empty(),
        )
    }
}
