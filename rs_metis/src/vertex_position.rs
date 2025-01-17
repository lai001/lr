use std::hash::Hash;

#[derive(Debug, Clone, Copy)]
pub struct VertexPosition {
    pub p: glam::Vec3,
}

impl VertexPosition {
    pub fn new(p: glam::Vec3) -> Self {
        Self { p }
    }
}

impl Eq for VertexPosition {}

impl Hash for VertexPosition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.p.x.to_bits().hash(state);
        self.p.y.to_bits().hash(state);
        self.p.z.to_bits().hash(state);
    }
}

impl PartialEq for VertexPosition {
    fn eq(&self, other: &Self) -> bool {
        self.p == other.p
    }
}
