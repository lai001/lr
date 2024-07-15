use v8::{Platform, SharedRef};

pub(crate) struct PlatformWrapper {
    pub(crate) platform: SharedRef<Platform>,
}

impl PlatformWrapper {
    pub(crate) fn new() -> PlatformWrapper {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::set_flags_from_string("--no_freeze_flags_after_init --expose-gc");
        v8::V8::initialize_platform(platform.clone());
        v8::V8::initialize();
        v8::cppgc::initalize_process(platform.clone());
        PlatformWrapper { platform }
    }
}

impl Drop for PlatformWrapper {
    fn drop(&mut self) {
        unsafe {
            v8::cppgc::shutdown_process();
            v8::V8::dispose();
        }
        v8::V8::dispose_platform();
    }
}
