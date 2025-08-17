pub struct Enviroment {
    pub status_bar_height: i32,
}

#[jni_fn::jni_fn("com.lai001.lib.lrjni.Environment")]
pub fn newEnvironment(_: jni::JNIEnv, _: jni::objects::JClass) -> jni::sys::jlong {
    let enviroment = Enviroment {
        status_bar_height: 0,
    };
    let enviroment = Box::new(enviroment);
    return Box::into_raw(enviroment) as jni::sys::jlong;
}

#[jni_fn::jni_fn("com.lai001.lib.lrjni.Environment")]
pub fn drop(_: jni::JNIEnv, _: jni::sys::jclass, enviroment: jni::sys::jlong) {
    unsafe {
        let _ = Box::from_raw(enviroment as *mut Enviroment);
    }
}

#[jni_fn::jni_fn("com.lai001.lib.lrjni.Environment")]
pub fn setStatusBarHeight(
    _: jni::JNIEnv,
    _: jni::sys::jclass,
    enviroment: jni::sys::jlong,
    status_bar_height: jni::sys::jint,
) {
    let enviroment = unsafe {
        (enviroment as *mut Enviroment)
            .as_mut()
            .expect("A valid pointer")
    };
    enviroment.status_bar_height = status_bar_height;
}
