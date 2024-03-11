use crate::convert::ConvertToAIString;
use russimp_sys::*;
use std::marker::PhantomData;

pub struct PropertyStore {
    c: *mut russimp_sys::aiPropertyStore,
    marker: PhantomData<()>,
}

impl Default for PropertyStore {
    fn default() -> PropertyStore {
        Self::new()
    }
}

impl PropertyStore {
    pub fn new() -> PropertyStore {
        let c = unsafe { aiCreatePropertyStore() };
        if c.is_null() {
            panic!();
        }
        PropertyStore {
            c,
            marker: PhantomData,
        }
    }

    pub fn get_mut(&mut self) -> &mut russimp_sys::aiPropertyStore {
        unsafe { self.c.as_mut().expect("Not null.") }
    }

    pub fn get(&self) -> &russimp_sys::aiPropertyStore {
        unsafe { self.c.as_ref().expect("Not null.") }
    }

    pub fn set_property_integer(&mut self, name: &str, value: i32) {
        unsafe { aiSetImportPropertyInteger(self.c, std::mem::transmute(name.as_ptr()), value) }
    }

    pub fn set_property_bool(&mut self, name: &str, value: bool) {
        unsafe {
            aiSetImportPropertyInteger(
                self.c,
                std::mem::transmute(name.as_ptr()),
                if value { 1 } else { 0 },
            )
        }
    }

    pub fn set_property_float(&mut self, name: &str, value: f32) {
        unsafe { aiSetImportPropertyFloat(self.c, std::mem::transmute(name.as_ptr()), value) }
    }

    pub fn set_property_string(&mut self, name: &str, value: &str) {
        let value = value.to_ai_string();
        unsafe { aiSetImportPropertyString(self.c, std::mem::transmute(name.as_ptr()), &value) }
    }
}

impl Drop for PropertyStore {
    fn drop(&mut self) {
        if self.c.is_null() {
            panic!();
        }
        unsafe { aiReleasePropertyStore(self.c) }
    }
}
