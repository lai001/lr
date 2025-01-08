use crate::edge::{Edge, TriangleEdge};
use std::collections::{HashMap, HashSet};

pub type GraphVertexIndex = u32;
pub type MeshVertexIndex = u32;
pub type TriangleIndex = u32;

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

#[derive(Default)]
pub struct Triangle {
    indices: [MeshVertexIndex; 3],
}

impl Triangle {
    fn get_edges(&self) -> [Edge; 3] {
        [
            Edge {
                v0: self.indices[0],
                v1: self.indices[1],
            },
            Edge {
                v0: self.indices[1],
                v1: self.indices[2],
            },
            Edge {
                v0: self.indices[2],
                v1: self.indices[0],
            },
        ]
    }

    pub fn get_indices(&self) -> &[MeshVertexIndex; 3] {
        &self.indices
    }
}

#[derive(Default)]
pub struct TriangleGraph {
    triangles: Vec<Triangle>,
    adjoin_triangles: Vec<HashSet<TriangleIndex>>,
    edges: HashSet<TriangleEdge>,
}

impl TriangleGraph {
    pub fn new(indices: &[MeshVertexIndex] /*, vertices: &[glam::Vec3]*/) -> TriangleGraph {
        assert!(indices.len() > 0);
        // assert!(vertices.len() > 0);
        assert_eq!(indices.len() % 3, 0);

        let mut triangles: Vec<Triangle> = Vec::with_capacity(indices.len() / 3);
        for triangle_indices in indices.chunks_exact(3) {
            let triangle = Triangle {
                indices: unsafe { triangle_indices.try_into().unwrap_unchecked() },
            };
            triangles.push(triangle);
        }

        let mut vertex_edges: HashMap<Edge, HashSet<usize>> = HashMap::new();
        for (i, triangle) in triangles.iter().enumerate() {
            let edges = triangle.get_edges();
            for edge in edges {
                vertex_edges.entry(edge).or_default().insert(i);
            }
        }

        let mut adjoin_triangles: Vec<HashSet<TriangleIndex>> =
            vec![HashSet::new(); triangles.len()];
        for (i, triangle) in triangles.iter().enumerate() {
            let edges = triangle.get_edges();
            for edge in edges {
                let mut triangle_indices = vertex_edges
                    .get(&edge)
                    .expect(&format!("Not null, {:?}", edge))
                    .clone();
                assert!(triangle_indices.contains(&i));
                triangle_indices.remove(&i);
                assert!(
                    triangle_indices.len() <= 1,
                    "An edge can not belong to more than two trignales"
                );
                let value: Option<u32> = (|| {
                    for i in triangle_indices {
                        return Some(i as TriangleIndex);
                    }
                    return None;
                })();
                if let Some(value) = value {
                    adjoin_triangles[i].insert(value);
                }
            }
        }

        let mut edges: HashSet<TriangleEdge> = HashSet::new();
        for (v0, adjoin_triangle) in adjoin_triangles.iter().enumerate() {
            for v1 in adjoin_triangle.clone() {
                let edge = TriangleEdge { v0: v0 as u32, v1 };
                edges.insert(edge);
            }
        }

        TriangleGraph {
            triangles,
            adjoin_triangles,
            edges,
        }
    }

    pub fn get_graph_vertices_len(&self) -> u32 {
        self.adjoin_triangles.len() as u32
    }

    pub fn get_graph_edges_len(&self) -> u32 {
        self.edges.len() as u32
    }

    pub fn get_triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn write_to_file(
        &self,
        output_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<()> {
        let mut content = String::new();
        content.push_str(&format!(
            "{} {}\n",
            self.get_graph_vertices_len(),
            self.get_graph_edges_len()
        ));
        for indices in &self.adjoin_triangles {
            let line: String = indices
                .iter()
                .map(|x| (x + 1).to_string())
                .collect::<Vec<String>>()
                .join(" ");
            content.push_str(&format!("{}\n", line));
        }
        std::fs::create_dir_all(
            output_path
                .as_ref()
                .parent()
                .ok_or(crate::error::Error::Other(Some(format!("No parent"))))?,
        )
        .map_err(|err| crate::error::Error::IO(err, None))?;
        if output_path.as_ref().exists() {
            std::fs::remove_file(output_path.as_ref())
                .map_err(|err| crate::error::Error::IO(err, None))?;
        }
        std::fs::write(output_path, content).map_err(|err| crate::error::Error::IO(err, None))
    }
}
