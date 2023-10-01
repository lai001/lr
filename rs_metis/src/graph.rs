use crate::edge::Edge;
use std::collections::{HashMap, HashSet};

pub type GraphVertexIndex = u32;
pub type MeshVertexIndex = u32;

#[derive(Default)]
pub struct Graph {
    pub adjoin_indices: Vec<Vec<GraphVertexIndex>>,
    pub graph_index_map_indices: HashMap<GraphVertexIndex, Vec<MeshVertexIndex>>,
    pub edges: HashSet<Edge>,
}

impl Graph {
    pub fn get_num_vertices(&self) -> u32 {
        self.adjoin_indices.len() as u32
    }

    pub fn get_num_edges(&self) -> u32 {
        self.edges.len() as u32
    }
}
