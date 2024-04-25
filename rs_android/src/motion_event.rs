use std::collections::HashMap;

pub struct MotionEvent<'a> {
    env: jni::JNIEnv<'a>,
    event: jni::objects::JClass<'a>,
}

impl<'a> MotionEvent<'a> {
    pub fn new(env: jni::JNIEnv<'a>, event: jni::objects::JClass<'a>) -> MotionEvent<'a> {
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
            0 => EActionType::ActionDown,
            1 => EActionType::ActionUp,
            2 => EActionType::ActionMove,
            3 => EActionType::ActionCancel,
            4 => EActionType::ActionOutside,
            _ => {
                panic!("Unknow value {}", value);
            }
        }
    }
}
