pub enum EValue {
    I32(i32),
    String(String),
    F32(f32),
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
}

pub const RS_TEST_KEY: &str = "rs.test";
