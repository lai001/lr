use crate::graph::GraphVertexIndex;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Edge {
    pub v0: GraphVertexIndex,
    pub v1: GraphVertexIndex,
}

impl Edge {
    pub fn new(v0: GraphVertexIndex, v1: GraphVertexIndex) -> Edge {
        Edge { v0, v1 }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.v0 == other.v1 || self.v1 == other.v0
    }
}

impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.v0 ^ self.v1).hash(state);
    }
}
