use winit::{event::ElementState, keyboard::KeyCode};

pub struct KeyEvent<'a> {
    env: jni::JNIEnv<'a>,
    event: jni::objects::JObject<'a>,
}

impl<'a> KeyEvent<'a> {
    pub fn new(env: jni::JNIEnv<'a>, event: jni::objects::JObject<'a>) -> KeyEvent<'a> {
        KeyEvent { env, event }
    }

    pub fn is_alt_pressed(&mut self) -> bool {
        let result = self
            .env
            .call_method(&self.event, "isAltPressed", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn is_shift_pressed(&mut self) -> bool {
        let result = self
            .env
            .call_method(&self.event, "isShiftPressed", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn is_ctrl_pressed(&mut self) -> bool {
        let result = self
            .env
            .call_method(&self.event, "isCtrlPressed", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn is_caps_lock_on(&mut self) -> bool {
        let result = self
            .env
            .call_method(&self.event, "isCapsLockOn", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn is_num_lock_on(&mut self) -> bool {
        let result = self.env.call_method(&self.event, "isNumLockOn", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn is_scroll_lock_on(&mut self) -> bool {
        let result = self
            .env
            .call_method(&self.event, "isScrollLockOn", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn get_action(&mut self) -> i32 {
        let result = self.env.call_method(&self.event, "getAction", "()I", &[]);
        result.unwrap().i().unwrap()
    }

    pub fn is_canceled(&mut self) -> bool {
        let result = self.env.call_method(&self.event, "isCanceled", "()Z", &[]);
        result.unwrap().z().unwrap()
    }

    pub fn get_scan_code(&mut self) -> i32 {
        let result = self.env.call_method(&self.event, "getScanCode", "()I", &[]);
        result.unwrap().i().unwrap()
    }

    pub fn get_key_code(&mut self) -> i32 {
        let result = self.env.call_method(&self.event, "getKeyCode", "()I", &[]);
        result.unwrap().i().unwrap()
    }

    pub fn to_string(&mut self) -> String {
        let result = self
            .env
            .call_method(&self.event, "toString", "()Ljava/lang/String;", &[])
            .unwrap();
        let java_str = result.l().expect("A Java String object");
        self.env
            .get_string((&java_str).into())
            .expect("A Java String")
            .into()
    }
}

pub fn to_key_code(code: i32) -> Option<KeyCode> {
    match code as u32 {
        ndk_sys::AKEYCODE_0 => Some(KeyCode::Digit0),
        ndk_sys::AKEYCODE_1 => Some(KeyCode::Digit1),
        ndk_sys::AKEYCODE_2 => Some(KeyCode::Digit2),
        ndk_sys::AKEYCODE_3 => Some(KeyCode::Digit3),
        ndk_sys::AKEYCODE_4 => Some(KeyCode::Digit4),
        ndk_sys::AKEYCODE_5 => Some(KeyCode::Digit5),
        ndk_sys::AKEYCODE_6 => Some(KeyCode::Digit6),
        ndk_sys::AKEYCODE_7 => Some(KeyCode::Digit7),
        ndk_sys::AKEYCODE_8 => Some(KeyCode::Digit8),
        ndk_sys::AKEYCODE_9 => Some(KeyCode::Digit9),
        ndk_sys::AKEYCODE_A => Some(KeyCode::KeyA),
        ndk_sys::AKEYCODE_B => Some(KeyCode::KeyB),
        ndk_sys::AKEYCODE_C => Some(KeyCode::KeyC),
        ndk_sys::AKEYCODE_D => Some(KeyCode::KeyD),
        ndk_sys::AKEYCODE_E => Some(KeyCode::KeyE),
        ndk_sys::AKEYCODE_F => Some(KeyCode::KeyF),
        ndk_sys::AKEYCODE_G => Some(KeyCode::KeyG),
        ndk_sys::AKEYCODE_H => Some(KeyCode::KeyH),
        ndk_sys::AKEYCODE_I => Some(KeyCode::KeyI),
        ndk_sys::AKEYCODE_J => Some(KeyCode::KeyJ),
        ndk_sys::AKEYCODE_K => Some(KeyCode::KeyK),
        ndk_sys::AKEYCODE_L => Some(KeyCode::KeyL),
        ndk_sys::AKEYCODE_M => Some(KeyCode::KeyM),
        ndk_sys::AKEYCODE_N => Some(KeyCode::KeyN),
        ndk_sys::AKEYCODE_O => Some(KeyCode::KeyO),
        ndk_sys::AKEYCODE_P => Some(KeyCode::KeyP),
        ndk_sys::AKEYCODE_Q => Some(KeyCode::KeyQ),
        ndk_sys::AKEYCODE_R => Some(KeyCode::KeyR),
        ndk_sys::AKEYCODE_S => Some(KeyCode::KeyS),
        ndk_sys::AKEYCODE_T => Some(KeyCode::KeyT),
        ndk_sys::AKEYCODE_U => Some(KeyCode::KeyU),
        ndk_sys::AKEYCODE_V => Some(KeyCode::KeyV),
        ndk_sys::AKEYCODE_W => Some(KeyCode::KeyW),
        ndk_sys::AKEYCODE_X => Some(KeyCode::KeyX),
        ndk_sys::AKEYCODE_Y => Some(KeyCode::KeyY),
        ndk_sys::AKEYCODE_Z => Some(KeyCode::KeyZ),
        ndk_sys::AKEYCODE_F1 => Some(KeyCode::F1),
        ndk_sys::AKEYCODE_F2 => Some(KeyCode::F2),
        ndk_sys::AKEYCODE_F3 => Some(KeyCode::F3),
        ndk_sys::AKEYCODE_F4 => Some(KeyCode::F4),
        ndk_sys::AKEYCODE_F5 => Some(KeyCode::F5),
        ndk_sys::AKEYCODE_F6 => Some(KeyCode::F6),
        ndk_sys::AKEYCODE_F7 => Some(KeyCode::F7),
        ndk_sys::AKEYCODE_F8 => Some(KeyCode::F8),
        ndk_sys::AKEYCODE_F9 => Some(KeyCode::F9),
        ndk_sys::AKEYCODE_F10 => Some(KeyCode::F10),
        ndk_sys::AKEYCODE_F11 => Some(KeyCode::F11),
        ndk_sys::AKEYCODE_F12 => Some(KeyCode::F12),
        ndk_sys::AKEYCODE_ENTER => Some(KeyCode::Enter),
        ndk_sys::AKEYCODE_TAB => Some(KeyCode::Tab),
        ndk_sys::AKEYCODE_SPACE => Some(KeyCode::Space),
        ndk_sys::AKEYCODE_DPAD_UP => Some(KeyCode::ArrowUp),
        ndk_sys::AKEYCODE_DPAD_DOWN => Some(KeyCode::ArrowDown),
        ndk_sys::AKEYCODE_DPAD_LEFT => Some(KeyCode::ArrowLeft),
        ndk_sys::AKEYCODE_DPAD_RIGHT => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

pub fn to_element_state(action: i32) -> Option<ElementState> {
    match action as u32 {
        ndk_sys::AKEY_EVENT_ACTION_DOWN => Some(ElementState::Pressed),
        ndk_sys::AKEY_EVENT_ACTION_UP => Some(ElementState::Released),
        _ => None,
    }
}
