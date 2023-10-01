use crate::{graph::GraphVertexIndex, vec3_hash_wrapper::Vec3HashWrapper};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Edge {
    pub p0: Vec3HashWrapper,
    pub p1: Vec3HashWrapper,
    pub v0: GraphVertexIndex,
    pub v1: GraphVertexIndex,
}

impl Edge {
    pub fn new(
        p0: Vec3HashWrapper,
        p1: Vec3HashWrapper,
        v0: GraphVertexIndex,
        v1: GraphVertexIndex,
    ) -> Edge {
        Edge { p0, p1, v0, v1 }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.p0 == other.p0 && self.p1 == other.p1) || (self.p0 == other.p1 && self.p1 == other.p0)
    }
}

impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let x = self.p0.value.x.to_bits() ^ self.p1.value.x.to_bits();
        let y = self.p0.value.y.to_bits() ^ self.p1.value.y.to_bits();
        let z = self.p0.value.z.to_bits() ^ self.p1.value.z.to_bits();
        x.hash(state);
        y.hash(state);
        z.hash(state);
    }
}
