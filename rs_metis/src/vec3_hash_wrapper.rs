use std::hash::{Hash, Hasher};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Vec3HashWrapper {
    pub value: glam::Vec3,
}

impl Vec3HashWrapper {
    pub fn new(value: glam::Vec3) -> Vec3HashWrapper {
        Vec3HashWrapper { value }
    }
}

impl Hash for Vec3HashWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.x.to_bits().hash(state);
        self.value.y.to_bits().hash(state);
        self.value.z.to_bits().hash(state);
    }
}

impl Eq for Vec3HashWrapper {}
