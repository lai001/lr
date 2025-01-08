use crate::graph::{GraphVertexIndex, TriangleIndex};
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
        if self.v0 == other.v0 && self.v1 == other.v1 {
            return true;
        } else if self.v0 == other.v1 && self.v1 == other.v0 {
            return true;
        }
        return false;
    }
}

impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.v0 ^ self.v1).hash(state);
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct TriangleEdge {
    pub v0: TriangleIndex,
    pub v1: TriangleIndex,
}

impl TriangleEdge {
    pub fn new(v0: TriangleIndex, v1: TriangleIndex) -> TriangleEdge {
        TriangleEdge { v0, v1 }
    }
}

impl PartialEq for TriangleEdge {
    fn eq(&self, other: &Self) -> bool {
        if self.v0 == other.v0 && self.v1 == other.v1 {
            return true;
        } else if self.v0 == other.v1 && self.v1 == other.v0 {
            return true;
        }
        return false;
    }
}

impl Hash for TriangleEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.v0 ^ self.v1).hash(state);
    }
}
