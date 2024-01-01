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
}
