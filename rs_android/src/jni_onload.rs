// https://blog.devops.dev/rust-on-android-lessons-from-the-edge-aed31a4d7726
#[cfg(feature = "panic_hook")]
fn set_panic_hook() {
    const LINE_ENDING: &str = "\n";
    std::panic::set_hook(Box::new(move |panic| {
        let reason = if let Some(s) = panic.payload().downcast_ref::<&str>() {
            format!("{s}")
        } else if let Some(s) = panic.payload().downcast_ref::<String>() {
            format!("{s}")
        } else {
            format!("{:?}", panic)
        };
        let backtrace = backtrace::Backtrace::new();
        match panic.location() {
            Some(location) => {
                log::error!(
                    "{}, file: {}, line: {}, col: {}, backtrace: {}{:#?}",
                    reason,
                    location.file(),
                    location.line(),
                    location.column(),
                    LINE_ENDING,
                    backtrace,
                );
            }
            None => {
                log::error!("{}, backtrace: {}{:#?}", reason, LINE_ENDING, backtrace);
            }
        }
    }));
}

#[no_mangle]
extern "C" fn JNI_OnLoad(vm: jni::JavaVM, res: *mut std::os::raw::c_void) -> jni::sys::jint {
    let vm = vm.get_java_vm_pointer() as *mut std::ffi::c_void;
    unsafe {
        ndk_context::initialize_android_context(vm, res);
    }
    #[cfg(feature = "panic_hook")]
    set_panic_hook();
    jni::JNIVersion::V6.into()
}
