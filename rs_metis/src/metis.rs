use crate::{
    edge::Edge,
    graph::{Graph, GraphVertexIndex, MeshVertexIndex},
    vec3_hash_wrapper::Vec3HashWrapper,
};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    process::Command,
};

fn loop_range_next(value: u32, range: Range<u32>) -> u32 {
    range.start + (value + 1) % (range.end - range.start)
}

fn loop_range_triangle_next(value: u32) -> u32 {
    let start = value / 3 * 3;
    let end = start + 3;
    loop_range_next(value, start..end)
}

pub struct Metis {}

impl Metis {
    pub fn to_graph(indices: &[u32], vertices: &[glam::Vec3]) -> Graph {
        debug_assert_eq!(indices.len(), vertices.len());
        debug_assert_eq!(indices.len() % 3, 0);

        let mut vertex_map_indices: HashMap<Vec3HashWrapper, Vec<MeshVertexIndex>> = HashMap::new();
        for index in indices {
            let vertex = vertices[*index as usize];
            let vertex = Vec3HashWrapper::new(vertex);

            match vertex_map_indices.get_mut(&vertex) {
                Some(value) => {
                    value.push(*index);
                }
                None => {
                    let mut value: Vec<u32> = Vec::new();
                    value.push(*index);
                    vertex_map_indices.insert(vertex, value);
                }
            }
        }

        let mut vertex_map_adjoin_vertices: HashMap<Vec3HashWrapper, HashSet<Vec3HashWrapper>> =
            HashMap::new();

        for (vertex, indices) in vertex_map_indices.iter() {
            let mut adjoin_vertices: HashSet<Vec3HashWrapper> = HashSet::new();
            for index in indices {
                let adjoin_vertex_index = loop_range_triangle_next(*index);
                let adjoin_vertex = vertices[adjoin_vertex_index as usize];
                adjoin_vertices.insert(Vec3HashWrapper::new(adjoin_vertex));
                let adjoin_vertex_index = loop_range_triangle_next(adjoin_vertex_index);
                let adjoin_vertex = vertices[adjoin_vertex_index as usize];
                adjoin_vertices.insert(Vec3HashWrapper::new(adjoin_vertex));
            }
            vertex_map_adjoin_vertices.insert(*vertex, adjoin_vertices);
        }

        let graph_vertices = vertex_map_adjoin_vertices
            .keys()
            .map(|x| *x)
            .collect::<Vec<Vec3HashWrapper>>();
        let mut graph_vertex_map_graph_vertex_index: HashMap<Vec3HashWrapper, GraphVertexIndex> =
            HashMap::new();
        for (index, graph_vertex) in graph_vertices.iter().enumerate() {
            graph_vertex_map_graph_vertex_index.insert(*graph_vertex, (index + 1) as u32);
        }

        let mut adjoin_indices: Vec<Vec<GraphVertexIndex>> = Vec::new();

        for graph_vertex in graph_vertices.iter() {
            let adjoin_vertices = vertex_map_adjoin_vertices.get(graph_vertex).unwrap();
            let mut graph_vertex_indices: Vec<GraphVertexIndex> = Vec::new();
            for adjoin_vertex in adjoin_vertices {
                let graph_vertex_index = graph_vertex_map_graph_vertex_index
                    .get(adjoin_vertex)
                    .unwrap();
                graph_vertex_indices.push(*graph_vertex_index);
            }
            adjoin_indices.push(graph_vertex_indices);
        }

        let mut graph_index_map_indices: HashMap<GraphVertexIndex, Vec<MeshVertexIndex>> =
            HashMap::new();

        for (graph_vertex, graph_vertex_index) in graph_vertex_map_graph_vertex_index.iter() {
            let indices = vertex_map_indices.get(graph_vertex).unwrap();
            graph_index_map_indices.insert(graph_vertex_index.clone(), indices.clone());
        }

        let mut edges: HashSet<Edge> = HashSet::new();
        for (vertex, adjoin_vertices) in vertex_map_adjoin_vertices.iter() {
            for adjoin_vertex in adjoin_vertices.iter() {
                let dege = Edge::new(
                    *vertex,
                    *adjoin_vertex,
                    *graph_vertex_map_graph_vertex_index.get(vertex).unwrap(),
                    *graph_vertex_map_graph_vertex_index
                        .get(adjoin_vertex)
                        .unwrap(),
                );
                edges.insert(dege);
            }
        }

        Graph {
            adjoin_indices,
            graph_index_map_indices,
            edges,
        }
    }

    pub fn build_mesh_clusters(
        graph: &Graph,
        partition: &[Vec<GraphVertexIndex>],
    ) -> Vec<Vec<MeshVertexIndex>> {
        let mut cluster_indices: Vec<Vec<MeshVertexIndex>> = Vec::new();

        type PartID = u32;
        let mut quick_search: HashMap<GraphVertexIndex, PartID> = HashMap::new();
        for (part_id, sub_partition) in partition.iter().enumerate() {
            for graph_vertex_index in sub_partition {
                quick_search.insert(*graph_vertex_index, part_id as u32);
            }
        }

        for sub_partition in partition {
            let mut all_triangles: HashSet<(u32, u32, u32)> = HashSet::new();

            for graph_vertex_index in sub_partition {
                let sub_indices = graph
                    .graph_index_map_indices
                    .get(graph_vertex_index)
                    .unwrap()
                    .clone();
                let triangles =
                    sub_indices
                        .iter()
                        .fold(HashSet::<(u32, u32, u32)>::new(), |mut acc, x| {
                            let triangles = Self::fill_indices_triangle(*x);
                            acc.insert(triangles);
                            acc
                        });

                for triangle in triangles {
                    all_triangles.insert(triangle);
                }
            }

            let sub_indices = all_triangles.iter().fold(Vec::<u32>::new(), |mut acc, x| {
                acc.push(x.0);
                acc.push(x.1);
                acc.push(x.2);
                acc
            });

            cluster_indices.push(sub_indices);
        }
        cluster_indices
    }

    fn fill_indices_triangle(index: u32) -> (u32, u32, u32) {
        let start = index / 3 * 3;
        (start, start + 1, start + 2)
    }

    pub fn partition(gpmetis_program_path: &str, graph: &Graph, num_parts: u32) -> Vec<Vec<u32>> {
        let output_path = std::path::Path::new("./t.graph").to_path_buf();
        let binding = rs_foundation::absolute_path(output_path).unwrap();
        let output_path = binding.to_str().unwrap();
        Self::write_graph(graph, output_path);
        let partition = Self::internal_partition(gpmetis_program_path, output_path, num_parts);
        // let _ = std::fs::remove_file(output_path);

        let mut partition_ret: Vec<Vec<u32>> = vec![vec![]; num_parts as usize];

        for (index, item) in partition.iter().enumerate() {
            let value = partition_ret.get_mut(*item as usize).unwrap();
            value.push((index + 1) as u32);
        }

        partition_ret
    }

    fn internal_partition(
        gpmetis_program_path: &str,
        graph_file_path: &str,
        num_parts: u32,
    ) -> Vec<u32> {
        let mut cmd = Command::new(gpmetis_program_path);
        cmd.args([graph_file_path, &num_parts.to_string()]);
        let output = cmd.output();
        match output {
            Ok(msg) => {
                if msg.status.success() {
                    log::trace!("{}", String::from_utf8_lossy(&msg.stdout));
                } else {
                    log::trace!("{}", String::from_utf8_lossy(&msg.stderr));
                }
            }
            Err(error) => log::warn!("{}", error),
        }
        let file_name = std::path::Path::new(graph_file_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let file_name = format!("{}.part.{}", file_name, num_parts);

        let graph_partition_file_path = std::path::Path::new(graph_file_path)
            .parent()
            .unwrap()
            .join(file_name);
        let file = std::fs::File::open(graph_partition_file_path.clone()).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut partition: Vec<u32> = Vec::new();
        for line in std::io::BufRead::lines(reader) {
            if let Ok(line) = line {
                let value: u32 = line.trim().parse::<u32>().unwrap();
                partition.push(value);
            }
        }
        // let _ = std::fs::remove_file(graph_partition_file_path);
        partition
    }

    fn write_graph(graph: &Graph, output_path: &str) {
        let mut content = String::new();
        content.push_str(&format!(
            "{} {}\n",
            graph.get_num_vertices(),
            graph.get_num_edges()
        ));
        for indices in &graph.adjoin_indices {
            let line = indices
                .iter()
                .fold(String::new(), |acc, x| format!("{} {}", acc, x));
            content.push_str(&format!("{}\n", line));
        }
        let output_path = std::path::Path::new(&output_path);
        std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
        std::fs::write(output_path, content).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::bindings::*;
    use crate::metis::*;

    #[test]
    fn test_case() {
        let graph_t = unsafe { ::std::mem::zeroed::<graph_t>() };
    }
}
