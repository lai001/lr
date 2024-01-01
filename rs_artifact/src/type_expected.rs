use serde::de::Expected;
use std::fmt;

pub struct GlamVec4Expected;

impl Expected for GlamVec4Expected {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("glam::Vec4")
    }
}

pub struct GlamVec3Expected;

impl Expected for GlamVec3Expected {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("glam::Vec3")
    }
}

pub struct GlamVec2Expected;

impl Expected for GlamVec2Expected {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("glam::Vec2")
    }
}
