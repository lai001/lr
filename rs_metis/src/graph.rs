#[cfg(feature = "required_detail_edges")]
use crate::edge::TriangleEdge;
use crate::{
    edge::{Edge, VertexEdge},
    vertex_position::VertexPosition,
};
use rs_core_minimal::thread_pool::ThreadPool;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub type GraphVertexIndex = u32;
pub type MeshVertexIndex = u32;
pub type TriangleIndex = u32;

#[derive(Default, Serialize, Deserialize)]
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

#[derive(Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Triangle {
    indices: [MeshVertexIndex; 3],
    edges: [Edge; 3],
}

impl Triangle {
    pub fn new(indices: [MeshVertexIndex; 3]) -> Self {
        Self {
            indices,
            edges: [
                Edge {
                    v0: indices[0],
                    v1: indices[1],
                },
                Edge {
                    v0: indices[1],
                    v1: indices[2],
                },
                Edge {
                    v0: indices[2],
                    v1: indices[0],
                },
            ],
        }
    }

    fn get_edges(&self) -> &[Edge; 3] {
        &self.edges
    }

    pub fn get_indices(&self) -> &[MeshVertexIndex; 3] {
        &self.indices
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriangleGraph {
    triangles: Vec<Triangle>,
    adjoin_triangles: Vec<HashSet<TriangleIndex>>,
    #[cfg(feature = "required_detail_edges")]
    edges: HashSet<TriangleEdge>,
    #[cfg(not(feature = "required_detail_edges"))]
    edges_len: usize,
}

impl TriangleGraph {
    pub fn parallel_from_indexed_vertices(
        indices: &[u32],
        vertices: Arc<Vec<VertexPosition>>,
    ) -> TriangleGraph {
        assert!(vertices.len() > 0);
        assert!(indices.len() > 0);
        assert_eq!(indices.len() % 3, 0);

        let mut triangles: Vec<Triangle> = Vec::with_capacity(indices.len() / 3);
        for triangle_indices in indices.chunks_exact(3) {
            let triangle = Triangle::new(unsafe { triangle_indices.try_into().unwrap_unchecked() });
            triangles.push(triangle);
        }
        let triangles = Arc::new(triangles);

        let vertex_edges = parallel_make_vertex_edges(triangles.clone(), vertices.clone());
        let vertex_edges = Arc::new(vertex_edges);

        let adjoin_triangles = parallel_make_adjoin_triangles(
            triangles.clone(),
            vertices.clone(),
            vertex_edges.clone(),
        );
        let adjoin_triangles = Arc::new(adjoin_triangles);

        #[cfg(feature = "required_detail_edges")]
        let edges = parallel_make_edges(adjoin_triangles.to_vec());
        #[cfg(not(feature = "required_detail_edges"))]
        let edges_num = {
            let mut edges_num = 0;
            for item in adjoin_triangles.iter() {
                edges_num += item.len();
            }
            edges_num
        };

        // while Arc::strong_count(&triangles) != 1 {}
        // while Arc::strong_count(&adjoin_triangles) != 1 {}

        let triangles = Arc::try_unwrap(triangles);
        let adjoin_triangles = Arc::try_unwrap(adjoin_triangles);
        match (triangles, adjoin_triangles) {
            (Ok(triangles), Ok(adjoin_triangles)) => TriangleGraph {
                triangles,
                adjoin_triangles,
                #[cfg(feature = "required_detail_edges")]
                edges,
                #[cfg(not(feature = "required_detail_edges"))]
                edges_len: edges_num / 2,
            },
            _ => panic!(),
        }
    }

    pub fn from_cache(
        triangles: &[Triangle],
        adjoin_triangles: &[HashSet<TriangleIndex>],
        selection: &[usize],
    ) -> TriangleGraph {
        let selection = HashSet::<usize>::from_iter(selection.to_vec());

        let mut sub_triangles: Vec<Triangle> = Vec::with_capacity(selection.len());
        let mut sub_adjoin_triangles: Vec<HashSet<TriangleIndex>> =
            Vec::with_capacity(selection.len());

        let mut map: HashMap<u32, u32> = HashMap::new();
        for (i, index) in selection.iter().enumerate() {
            sub_triangles.push(triangles[*index]);
            map.insert(*index as u32, i as u32);
        }

        for (_, index) in selection.iter().enumerate() {
            let mut values = Vec::from_iter(adjoin_triangles[*index].clone());
            values.retain(|x| {
                return selection.contains(&(*x as usize));
            });
            for value in values.iter_mut() {
                *value = map[value];
            }
            sub_adjoin_triangles.push(HashSet::from_iter(values));
        }

        let mut edges_num = 0;
        for item in sub_adjoin_triangles.iter() {
            edges_num += item.len();
        }

        #[cfg(feature = "required_detail_edges")]
        let edges: Vec<TriangleEdge> = {
            let mut edges: Vec<TriangleEdge> = Vec::with_capacity(edges_num);
            {
                let _ = tracy_client::span!("Build edges");
                for (v0, adjoin_triangle) in sub_adjoin_triangles.iter().enumerate() {
                    for v1 in adjoin_triangle {
                        let edge = TriangleEdge {
                            v0: v0 as u32,
                            v1: *v1,
                        };
                        edges.push(edge);
                    }
                }
            }
            // let mut map: HashMap<u64, TriangleEdge> = HashMap::new();
            // for edge in edges {
            //     let compact = (edge.v0 as u64) << 32 | edge.v1 as u64;
            //     map.insert(compact, edge);
            // }
            edges.iter_mut().for_each(|edge| {
                if edge.v0 > edge.v1 {
                    std::mem::swap(&mut edge.v0, &mut edge.v1);
                }
            });
            edges.sort_unstable();
            edges.dedup();
            edges
        };

        TriangleGraph {
            triangles: sub_triangles,
            adjoin_triangles: sub_adjoin_triangles,
            #[cfg(feature = "required_detail_edges")]
            edges: HashSet::from_iter(edges),
            #[cfg(not(feature = "required_detail_edges"))]
            edges_len: edges_num / 2,
        }
    }

    pub fn from_indexed_vertices(indices: &[u32], vertices: &[VertexPosition]) -> TriangleGraph {
        let _ = tracy_client::span!();

        assert!(vertices.len() > 0);
        assert!(indices.len() > 0);
        assert_eq!(indices.len() % 3, 0);

        let mut triangles: Vec<Triangle> = Vec::with_capacity(indices.len() / 3);
        {
            let _ = tracy_client::span!("Build triangles");
            for triangle_indices in indices.chunks_exact(3) {
                let triangle =
                    Triangle::new(unsafe { triangle_indices.try_into().unwrap_unchecked() });
                triangles.push(triangle);
            }
        }

        let mut vertex_edges: HashMap<VertexEdge, HashSet<usize>> = HashMap::new();
        {
            let _ = tracy_client::span!("Build vertex edges");
            for (i, triangle) in triangles.iter().enumerate() {
                let edges = triangle.get_edges();
                for edge in edges {
                    let vertex_edge =
                        VertexEdge::new(vertices[edge.v0 as usize], vertices[edge.v1 as usize]);
                    vertex_edges.entry(vertex_edge).or_default().insert(i);
                }
            }
        }

        let mut adjoin_triangles: Vec<HashSet<TriangleIndex>> =
            vec![HashSet::new(); triangles.len()];
        {
            let _ = tracy_client::span!("Build adjoin triangles");
            for (i, triangle) in triangles.iter().enumerate() {
                let edges = triangle.get_edges();
                for edge in edges {
                    let vertex_edge =
                        VertexEdge::new(vertices[edge.v0 as usize], vertices[edge.v1 as usize]);
                    let triangle_indices = vertex_edges
                        .get(&vertex_edge)
                        .expect(&format!("Not null, {:?}", edge));
                    debug_assert!(triangle_indices.contains(&i));
                    for value in triangle_indices {
                        if value != &i {
                            adjoin_triangles[i].insert(*value as u32);
                        }
                    }
                }
            }
        }

        let mut edges_num = 0;
        for item in adjoin_triangles.iter() {
            edges_num += item.len();
        }

        #[cfg(feature = "required_detail_edges")]
        let edges: Vec<TriangleEdge> = {
            let mut edges: Vec<TriangleEdge> = Vec::with_capacity(edges_num);
            {
                let _ = tracy_client::span!("Build edges");
                for (v0, adjoin_triangle) in adjoin_triangles.iter().enumerate() {
                    for v1 in adjoin_triangle {
                        let edge = TriangleEdge {
                            v0: v0 as u32,
                            v1: *v1,
                        };
                        edges.push(edge);
                    }
                }
            }
            // let mut map: HashMap<u64, TriangleEdge> = HashMap::new();
            // for edge in edges {
            //     let compact = (edge.v0 as u64) << 32 | edge.v1 as u64;
            //     map.insert(compact, edge);
            // }
            edges.iter_mut().for_each(|edge| {
                if edge.v0 > edge.v1 {
                    std::mem::swap(&mut edge.v0, &mut edge.v1);
                }
            });
            edges.sort_unstable();
            edges.dedup();
            edges
        };

        TriangleGraph {
            triangles,
            adjoin_triangles,
            #[cfg(feature = "required_detail_edges")]
            edges: HashSet::from_iter(edges),
            #[cfg(not(feature = "required_detail_edges"))]
            edges_len: edges_num / 2,
        }
    }

    pub fn get_graph_vertices_len(&self) -> u32 {
        self.adjoin_triangles.len() as u32
    }

    pub fn get_graph_edges_len(&self) -> u32 {
        #[cfg(feature = "required_detail_edges")]
        return self.edges.len() as u32;
        #[cfg(not(feature = "required_detail_edges"))]
        return self.edges_len as u32;
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
        let error_path = Some(format!(
            "{:?}",
            output_path.as_ref().to_str().unwrap_or_default()
        ));
        std::fs::create_dir_all(
            output_path
                .as_ref()
                .parent()
                .ok_or(crate::error::Error::Other(Some(format!("No parent"))))?,
        )
        .map_err(|err| crate::error::Error::IO(err, error_path.clone()))?;
        if output_path.as_ref().exists() {
            std::fs::remove_file(output_path.as_ref())
                .map_err(|err| crate::error::Error::IO(err, error_path.clone()))?;
        }
        std::fs::write(output_path.as_ref(), content)
            .map_err(|err| crate::error::Error::IO(err, error_path.clone()))
    }

    pub fn write_debug_info_to_file(
        &self,
        output_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<()> {
        let mut contents = String::new();

        contents.push_str("--Triangles\n");
        for (i, triangle) in self.triangles.iter().enumerate() {
            contents.push_str(&format!("{} {:?}\n", i, triangle.indices));
        }

        #[cfg(feature = "required_detail_edges")]
        {
            contents.push_str("--Edges\n");
            for edge in self.edges.iter() {
                contents.push_str(&format!("{} {}\n", edge.v0, edge.v1));
            }
        }

        contents.push_str("--Adjoin triangles\n");
        for (i, adjoin_triangle) in self.adjoin_triangles.iter().enumerate() {
            contents.push_str(&format!("{} {:?}\n", i, adjoin_triangle));
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
        std::fs::write(output_path, contents).map_err(|err| crate::error::Error::IO(err, None))
    }

    pub fn get_adjoin_triangles(&self) -> &[HashSet<u32>] {
        &self.adjoin_triangles
    }
}

fn parallel_make_vertex_edges(
    triangles: Arc<Vec<Triangle>>,
    vertices: Arc<Vec<VertexPosition>>,
) -> HashMap<VertexEdge, HashSet<usize>> {
    let count = std::thread::available_parallelism().unwrap().get();

    let binding = (0..triangles.len()).collect::<Vec<usize>>();
    let batchs = binding.chunks(triangles.len() / count);
    let batchs_len = batchs.len();

    let mut vertex_edges: HashMap<VertexEdge, HashSet<usize>> = HashMap::with_capacity(batchs_len);

    let (sender, receiver) = std::sync::mpsc::channel();

    for batch in batchs {
        ThreadPool::global().spawn({
            let batch = batch.to_vec();
            let triangles = triangles.clone();
            let vertices = vertices.clone();
            let sender = sender.clone();
            move || {
                let mut vertex_edges: HashMap<VertexEdge, HashSet<usize>> =
                    HashMap::with_capacity(batch.len());
                for i in batch {
                    let triangle = &triangles[i];
                    let edges = triangle.get_edges();
                    for edge in edges {
                        let vertex_edge =
                            VertexEdge::new(vertices[edge.v0 as usize], vertices[edge.v1 as usize]);
                        vertex_edges.entry(vertex_edge).or_default().insert(i);
                    }
                }
                sender.send(vertex_edges).unwrap();
            }
        });
    }

    for _ in 0..batchs_len {
        match receiver.recv() {
            Ok(task_result) => {
                for (k, v) in task_result {
                    vertex_edges.entry(k).or_default().extend(v);
                }
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    vertex_edges
}

fn parallel_make_adjoin_triangles(
    triangles: Arc<Vec<Triangle>>,
    vertices: Arc<Vec<VertexPosition>>,
    vertex_edges: Arc<HashMap<VertexEdge, HashSet<usize>>>,
) -> Vec<HashSet<TriangleIndex>> {
    struct TaskResult {
        offset: usize,
        adjoin_triangles: Vec<HashSet<TriangleIndex>>,
    }

    let mut adjoin_triangles: Vec<HashSet<TriangleIndex>> = Vec::with_capacity(triangles.len());

    let count = std::thread::available_parallelism().unwrap().get();

    let binding = (0..triangles.len()).collect::<Vec<usize>>();
    let batchs = binding.chunks(triangles.len() / count);
    let batchs_len = batchs.len();

    let (sender, receiver) = std::sync::mpsc::channel();
    for batch in batchs {
        ThreadPool::global().spawn({
            let mut batch = batch.to_vec();
            let triangles = triangles.clone();
            let vertices = vertices.clone();
            let sender = sender.clone();
            let vertex_edges = vertex_edges.clone();
            move || {
                let mut adjoin_triangles: Vec<HashSet<TriangleIndex>> =
                    vec![HashSet::new(); batch.len()];
                let offset = batch[0];
                for (i, bi) in batch.drain(..).enumerate() {
                    let triangle = &triangles[bi];
                    let edges = triangle.get_edges();
                    for edge in edges {
                        let vertex_edge =
                            VertexEdge::new(vertices[edge.v0 as usize], vertices[edge.v1 as usize]);

                        let mut triangle_indices = vertex_edges
                            .get(&vertex_edge)
                            .expect(&format!("Not null, {:?}", edge))
                            .clone();
                        debug_assert!(triangle_indices.contains(&bi));
                        triangle_indices.remove(&bi);
                        for value in triangle_indices {
                            adjoin_triangles[i].insert(value as u32);
                        }
                    }
                }
                let task_result = TaskResult {
                    offset,
                    adjoin_triangles,
                };
                sender.send(task_result).unwrap();
            }
        });
    }

    let mut task_results: Vec<TaskResult> = Vec::with_capacity(batchs_len);

    for _ in 0..batchs_len {
        match receiver.recv() {
            Ok(task_result) => {
                task_results.push(task_result);
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    task_results.sort_by(|a, b| a.offset.cmp(&b.offset));

    for task_result in task_results {
        adjoin_triangles.extend(task_result.adjoin_triangles);
    }

    adjoin_triangles
}

#[cfg(feature = "required_detail_edges")]
fn parallel_make_edges(adjoin_triangles: Vec<HashSet<u32>>) -> HashSet<TriangleEdge> {
    let mut edges: HashSet<TriangleEdge> = HashSet::new();

    let count = std::thread::available_parallelism().unwrap().get();

    let binding = (0..adjoin_triangles.len()).collect::<Vec<usize>>();
    let batchs = binding.chunks(adjoin_triangles.len() / count);
    let batchs_len = batchs.len();

    let (sender, receiver) = std::sync::mpsc::channel();
    for batch in batchs {
        ThreadPool::global().spawn({
            let batch = batch.to_vec();
            let len = batch.len();
            let sender = sender.clone();
            let adjoin_triangles = adjoin_triangles[batch[0]..=batch[len - 1]].to_vec();
            move || {
                let mut edges: HashSet<TriangleEdge> = HashSet::new();
                assert_eq!(batch.len(), adjoin_triangles.len());
                for (triangle_index, triangles) in std::iter::zip(batch, adjoin_triangles) {
                    for v1 in triangles {
                        let edge = TriangleEdge {
                            v0: triangle_index as u32,
                            v1,
                        };
                        edges.insert(edge);
                    }
                }
                sender.send(edges).unwrap();
            }
        });
    }

    for _ in 0..batchs_len {
        match receiver.recv() {
            Ok(task_result) => {
                edges.extend(task_result);
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    edges
}

#[cfg(test)]
mod tests {
    use super::{MeshVertexIndex, TriangleGraph};
    use crate::{edge::TriangleEdge, vertex_position::VertexPosition};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_case() {
        let indices: Vec<MeshVertexIndex> = vec![0, 1, 3, 1, 2, 3];
        let vertices = vec![
            VertexPosition::new(glam::vec3(0.0, 0.0, 0.0)),
            VertexPosition::new(glam::vec3(1.0, 0.0, 0.0)),
            VertexPosition::new(glam::vec3(1.0, 0.0, 1.0)),
            VertexPosition::new(glam::vec3(0.0, 0.0, 1.0)),
        ];
        let triangle_graph = TriangleGraph::from_indexed_vertices(&indices, &vertices);
        assert_eq!(triangle_graph.get_graph_vertices_len(), 2);
        assert_eq!(triangle_graph.get_graph_edges_len(), 1);
        for item in triangle_graph.get_adjoin_triangles() {
            assert_eq!(item.len(), 1);
        }
    }

    #[test]
    fn dedup_test() {
        let mut edges: Vec<TriangleEdge> = vec![
            TriangleEdge { v0: 100, v1: 1 },
            TriangleEdge { v0: 1, v1: 890 },
            TriangleEdge { v0: 2, v1: 1230 },
            TriangleEdge { v0: 660, v1: 24 },
            TriangleEdge { v0: 24, v1: 660 },
            TriangleEdge { v0: 300, v1: 60 },
        ];
        edges.iter_mut().for_each(|edge| {
            if edge.v0 > edge.v1 {
                std::mem::swap(&mut edge.v0, &mut edge.v1);
            }
        });
        edges.dedup();

        let mut map: HashMap<u32, TriangleEdge> = HashMap::new();
        for edge in edges.iter() {
            let k = edge.v0 ^ edge.v1;
            map.insert(k, edge.clone());
        }

        let len = edges.len();
        let edges_set: HashSet<TriangleEdge> = HashSet::from_iter(edges);
        assert_eq!(len, edges_set.len());
        assert_eq!(map.len(), edges_set.len());
    }
}
