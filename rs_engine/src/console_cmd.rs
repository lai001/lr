pub enum EValue {
    I32(i32),
    String(String),
    F32(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
}

pub struct ConsoleCmd {
    pub key: String,
    pub value: EValue,
}

impl ConsoleCmd {
    pub fn get_i32_value(&self) -> i32 {
        match self.value {
            EValue::I32(value) => value,
            _ => todo!(),
        }
    }

    pub fn get_f32_value(&self) -> f32 {
        match self.value {
            EValue::F32(value) => value,
            _ => todo!(),
        }
    }

    pub fn get_string_value(&self) -> &str {
        match &self.value {
            EValue::String(value) => &value,
            _ => todo!(),
        }
    }

    pub fn get_vec2_value(&self) -> &glam::Vec2 {
        match &self.value {
            EValue::Vec2(value) => &value,
            _ => todo!(),
        }
    }

    pub fn get_vec3_value(&self) -> &glam::Vec3 {
        match &self.value {
            EValue::Vec3(value) => &value,
            _ => todo!(),
        }
    }

    pub fn get_vec4_value(&self) -> &glam::Vec4 {
        match &self.value {
            EValue::Vec4(value) => &value,
            _ => todo!(),
        }
    }
}

pub const RS_TEST_KEY: &str = "rs.test";
