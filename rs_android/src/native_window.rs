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

impl raw_window_handle::HasWindowHandle for NativeWindow {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let handle = raw_window_handle::AndroidNdkWindowHandle::new(
            std::ptr::NonNull::new(self.a_native_window as *mut _ as *mut core::ffi::c_void)
                .unwrap(),
        );
        Ok(unsafe {
            raw_window_handle::WindowHandle::borrow_raw(
                raw_window_handle::RawWindowHandle::AndroidNdk(handle),
            )
        })
    }
}

impl raw_window_handle::HasDisplayHandle for NativeWindow {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        unsafe {
            Ok(raw_window_handle::DisplayHandle::borrow_raw(
                raw_window_handle::RawDisplayHandle::Android(
                    raw_window_handle::AndroidDisplayHandle::new(),
                ),
            ))
        }
    }
}
