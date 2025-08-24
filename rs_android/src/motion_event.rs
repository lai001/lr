use std::{collections::HashMap, fmt::Debug};

pub const ACTION_BUTTON_PRESS: i32 = 11;
pub const ACTION_BUTTON_RELEASE: i32 = 12;
pub const ACTION_CANCEL: i32 = 3;
pub const ACTION_DOWN: i32 = 0;
pub const ACTION_HOVER_ENTER: i32 = 9;
pub const ACTION_HOVER_EXIT: i32 = 10;
pub const ACTION_HOVER_MOVE: i32 = 7;
pub const ACTION_MASK: i32 = 255;
pub const ACTION_MOVE: i32 = 2;
pub const ACTION_OUTSIDE: i32 = 4;
pub const ACTION_POINTER_DOWN: i32 = 5;
pub const ACTION_POINTER_INDEX_MASK: i32 = 65280;
pub const ACTION_POINTER_INDEX_SHIFT: i32 = 8;
pub const ACTION_POINTER_UP: i32 = 6;
pub const ACTION_SCROLL: i32 = 8;
pub const ACTION_UP: i32 = 1;
pub const AXIS_BRAKE: i32 = 23;
pub const AXIS_DISTANCE: i32 = 24;
pub const AXIS_GAS: i32 = 22;
pub const AXIS_GENERIC_1: i32 = 32;
pub const AXIS_GENERIC_10: i32 = 41;
pub const AXIS_GENERIC_11: i32 = 42;
pub const AXIS_GENERIC_12: i32 = 43;
pub const AXIS_GENERIC_13: i32 = 44;
pub const AXIS_GENERIC_14: i32 = 45;
pub const AXIS_GENERIC_15: i32 = 46;
pub const AXIS_GENERIC_16: i32 = 47;
pub const AXIS_GENERIC_2: i32 = 33;
pub const AXIS_GENERIC_3: i32 = 34;
pub const AXIS_GENERIC_4: i32 = 35;
pub const AXIS_GENERIC_5: i32 = 36;
pub const AXIS_GENERIC_6: i32 = 37;
pub const AXIS_GENERIC_7: i32 = 38;
pub const AXIS_GENERIC_8: i32 = 39;
pub const AXIS_GENERIC_9: i32 = 40;
pub const AXIS_GESTURE_PINCH_SCALE_FACTOR: i32 = 52;
pub const AXIS_GESTURE_SCROLL_X_DISTANCE: i32 = 50;
pub const AXIS_GESTURE_SCROLL_Y_DISTANCE: i32 = 51;
pub const AXIS_GESTURE_X_OFFSET: i32 = 48;
pub const AXIS_GESTURE_Y_OFFSET: i32 = 49;
pub const AXIS_HAT_X: i32 = 15;
pub const AXIS_HAT_Y: i32 = 16;
pub const AXIS_HSCROLL: i32 = 10;
pub const AXIS_LTRIGGER: i32 = 17;
pub const AXIS_ORIENTATION: i32 = 8;
pub const AXIS_PRESSURE: i32 = 2;
pub const AXIS_RELATIVE_X: i32 = 27;
pub const AXIS_RELATIVE_Y: i32 = 28;
pub const AXIS_RTRIGGER: i32 = 18;
pub const AXIS_RUDDER: i32 = 20;
pub const AXIS_RX: i32 = 12;
pub const AXIS_RY: i32 = 13;
pub const AXIS_RZ: i32 = 14;
pub const AXIS_SCROLL: i32 = 26;
pub const AXIS_SIZE: i32 = 3;
pub const AXIS_THROTTLE: i32 = 19;
pub const AXIS_TILT: i32 = 25;
pub const AXIS_TOOL_MAJOR: i32 = 6;
pub const AXIS_TOOL_MINOR: i32 = 7;
pub const AXIS_TOUCH_MAJOR: i32 = 4;
pub const AXIS_TOUCH_MINOR: i32 = 5;
pub const AXIS_VSCROLL: i32 = 9;
pub const AXIS_WHEEL: i32 = 21;
pub const AXIS_X: i32 = 0;
pub const AXIS_Y: i32 = 1;
pub const AXIS_Z: i32 = 11;
pub const BUTTON_BACK: i32 = 8;
pub const BUTTON_FORWARD: i32 = 16;
pub const BUTTON_PRIMARY: i32 = 1;
pub const BUTTON_SECONDARY: i32 = 2;
pub const BUTTON_STYLUS_PRIMARY: i32 = 32;
pub const BUTTON_STYLUS_SECONDARY: i32 = 64;
pub const BUTTON_TERTIARY: i32 = 4;

pub struct MotionEvent<'a> {
    env: jni::JNIEnv<'a>,
    event: jni::objects::JObject<'a>,
}

impl<'a> MotionEvent<'a> {
    pub fn new(env: jni::JNIEnv<'a>, event: jni::objects::JObject<'a>) -> MotionEvent<'a> {
        MotionEvent { env, event }
    }

    pub fn get_action(&mut self) -> i32 {
        let result = self.env.call_method(&self.event, "getAction", "()I", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Int(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_x(&mut self) -> f32 {
        let result = self.env.call_method(&self.event, "getX", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_x_with_pointer_index(&mut self, pointer_index: i32) -> f32 {
        let result = self.env.call_method(
            &self.event,
            "getX",
            "(I)F",
            &[jni::objects::JValueGen::Int(pointer_index)],
        );
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_y(&mut self) -> f32 {
        let result = self.env.call_method(&self.event, "getY", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_y_with_pointer_index(&mut self, pointer_index: i32) -> f32 {
        let result = self.env.call_method(
            &self.event,
            "getY",
            "(I)F",
            &[jni::objects::JValueGen::Int(pointer_index)],
        );
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_pressure(&mut self) -> f32 {
        let result = self.env.call_method(&self.event, "getPressure", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_size(&mut self) -> f32 {
        let result = self.env.call_method(&self.event, "getSize", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_touch_major(&mut self) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getTouchMajor", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_touch_minor(&mut self) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getTouchMinor", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_orientation(&mut self) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getOrientation", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_pointer_count(&mut self) -> i32 {
        let result = self
            .env
            .call_method(&self.event, "getPointerCount", "()I", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Int(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_device_id(&mut self) -> i32 {
        let result = self.env.call_method(&self.event, "getDeviceId", "()I", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Int(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_x_precision(&mut self) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getXPrecision", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_y_precision(&mut self) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getYPrecision", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_raw_x(&mut self) -> f32 {
        let result = self.env.call_method(&self.event, "getRawX", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_raw_y(&mut self) -> f32 {
        let result = self.env.call_method(&self.event, "getRawY", "()F", &[]);
        if let Ok(value) = result {
            if let jni::objects::JValueGen::Float(value) = value {
                return value;
            }
        }
        panic!()
    }

    pub fn get_axis_value(&mut self, axis: i32, pointer_index: i32) -> f32 {
        let result = self.env.call_method(
            &self.event,
            "getAxisValue",
            "(II)F",
            &[axis.into(), pointer_index.into()],
        );
        result.unwrap().f().unwrap()
    }

    pub fn get_history_size(&mut self) -> i32 {
        let result = self
            .env
            .call_method(&self.event, "getHistorySize", "()I", &[]);
        result.unwrap().i().unwrap()
    }

    pub fn get_historical_x(&mut self, pos: i32) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getHistoricalX", "(I)F", &[pos.into()]);
        result.unwrap().f().unwrap()
    }

    pub fn get_historical_y(&mut self, pos: i32) -> f32 {
        let result = self
            .env
            .call_method(&self.event, "getHistoricalY", "(I)F", &[pos.into()]);
        result.unwrap().f().unwrap()
    }

    pub fn get_action_masked(&mut self) -> i32 {
        let result = self
            .env
            .call_method(&self.event, "getActionMasked", "()I", &[]);
        result.unwrap().i().unwrap()
    }

    pub fn get_action_index(&mut self) -> i32 {
        let result = self
            .env
            .call_method(&self.event, "getActionIndex", "()I", &[]);
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

    pub fn to_geometry(&mut self) -> Geometry {
        let mut x_with_pointer_index: HashMap<i32, f32> = HashMap::new();
        let mut y_with_pointer_index: HashMap<i32, f32> = HashMap::new();
        for pointer_index in 0..self.get_pointer_count() {
            x_with_pointer_index
                .insert(pointer_index, self.get_x_with_pointer_index(pointer_index));
            y_with_pointer_index
                .insert(pointer_index, self.get_y_with_pointer_index(pointer_index));
        }

        Geometry {
            raw_x: self.get_raw_x(),
            raw_y: self.get_raw_y(),
            y_precision: self.get_y_precision(),
            x_precision: self.get_x_precision(),
            device_id: self.get_device_id(),
            pointer_count: self.get_pointer_count(),
            orientation: self.get_orientation(),
            touch_minor: self.get_touch_minor(),
            touch_major: self.get_touch_major(),
            size: self.get_size(),
            pressure: self.get_pressure(),
            x: self.get_x(),
            y: self.get_y(),
            x_with_pointer_index,
            y_with_pointer_index,
            action: EActionType::from_raw(self.get_action()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub raw_x: f32,
    pub raw_y: f32,
    pub y_precision: f32,
    pub x_precision: f32,
    pub device_id: i32,
    pub pointer_count: i32,
    pub orientation: f32,
    pub touch_minor: f32,
    pub touch_major: f32,
    pub size: f32,
    pub pressure: f32,
    pub x: f32,
    pub y: f32,
    pub x_with_pointer_index: HashMap<i32, f32>,
    pub y_with_pointer_index: HashMap<i32, f32>,
    pub action: EActionType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EActionType {
    ActionUp,
    ActionMove,
    ActionDown,
    ActionCancel,
    ActionOutside,
}

impl EActionType {
    pub fn from_raw(value: i32) -> EActionType {
        match value {
            ACTION_DOWN => EActionType::ActionDown,
            ACTION_UP => EActionType::ActionUp,
            ACTION_MOVE => EActionType::ActionMove,
            ACTION_CANCEL => EActionType::ActionCancel,
            ACTION_OUTSIDE => EActionType::ActionOutside,
            _ => {
                panic!("Unknow value {}", value);
            }
        }
    }
}
