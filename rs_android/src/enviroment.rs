pub struct Enviroment {
    pub status_bar_height: i32,
}

impl Enviroment {
    pub fn new(env: &mut jni::JNIEnv, android_enviroment: &mut jni::objects::JClass) -> Enviroment {
        Enviroment {
            status_bar_height: Self::get_status_bar_height(env, android_enviroment),
        }
    }

    fn get_status_bar_height(
        env: &mut jni::JNIEnv,
        android_enviroment: &mut jni::objects::JClass,
    ) -> i32 {
        let result = env.get_field(android_enviroment, "statusBarHeight", "I");
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Int(value) = value {
                return value;
            }
        }
        panic!()
    }
}
