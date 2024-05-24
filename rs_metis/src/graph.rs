use crate::edge::Edge;
use std::collections::HashSet;

pub type GraphVertexIndex = u32;
pub type MeshVertexIndex = u32;

#[derive(Default)]
pub struct Graph {
    pub adjoin_indices: Vec<HashSet<GraphVertexIndex>>,
    pub edges: HashSet<Edge>,
    pub graph_vertex_associated_indices: Vec<HashSet<usize>>,
}

impl Graph {
    pub fn get_num_vertices(&self) -> u32 {
        self.adjoin_indices.len() as u32
    }

    pub fn get_num_edges(&self) -> u32 {
        self.edges.len() as u32
    }
}
