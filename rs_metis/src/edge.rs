use crate::{
    graph::{GraphVertexIndex, TriangleIndex},
    vertex_position::VertexPosition,
};
use std::hash::{DefaultHasher, Hash, Hasher};

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

#[derive(Debug, Clone, Copy, Eq)]
pub struct VertexEdge {
    pub v0: VertexPosition,
    pub v1: VertexPosition,
}

impl VertexEdge {
    pub fn new(v0: VertexPosition, v1: VertexPosition) -> VertexEdge {
        VertexEdge { v0, v1 }
    }

    pub fn new2(v0: glam::Vec3, v1: glam::Vec3) -> VertexEdge {
        Self::new(VertexPosition::new(v0), VertexPosition::new(v1))
    }
}

impl PartialEq for VertexEdge {
    fn eq(&self, other: &Self) -> bool {
        if self.v0 == other.v0 && self.v1 == other.v1 {
            return true;
        } else if self.v0 == other.v1 && self.v1 == other.v0 {
            return true;
        }
        return false;
    }
}

impl Hash for VertexEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut a = DefaultHasher::new();
        let mut b = DefaultHasher::new();
        self.v0.hash(&mut a);
        self.v1.hash(&mut b);
        (a.finish() ^ b.finish()).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::edge::{TriangleEdge, VertexEdge};

    #[test]
    fn test_case() {
        let t1 = TriangleEdge::new(10, 20);
        let t2 = TriangleEdge::new(20, 10);
        let t3 = TriangleEdge::new(21, 10);
        assert_eq!(t1, t2);
        assert_ne!(t2, t3);
    }

    #[test]
    fn test_case1() {
        let t1 = VertexEdge::new2(glam::vec3(0.0, 1.0, 2.0), glam::vec3(10.0, 1.0, 2.0));
        let t2 = VertexEdge::new2(glam::vec3(10.0, 1.0, 2.0), glam::vec3(0.0, 1.0, 2.0));
        let t3 = VertexEdge::new2(glam::vec3(10.0, 1.0, 2.0), glam::vec3(11.0, 1.0, 2.0));
        assert_eq!(t1, t2);
        assert_ne!(t2, t3);
    }
}
