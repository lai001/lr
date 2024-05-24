use crate::{
    edge::Edge,
    graph::{Graph, GraphVertexIndex},
};
use std::{collections::HashSet, ops::Range, process::Command};

fn loop_range_next(value: usize, range: Range<usize>) -> usize {
    range.start + (value + 1) % (range.end - range.start)
}

fn loop_range_triangle_next(value: usize) -> usize {
    let start = value / 3 * 3;
    let end = start + 3;
    loop_range_next(value, start..end)
}

pub struct Metis {}

impl Metis {
    fn find_other_two_vertex_indices(at: usize, indices: &[u32]) -> [u32; 2] {
        let next_at = loop_range_triangle_next(at);
        let first = indices[next_at];
        let next_at = loop_range_triangle_next(next_at);
        let second = indices[next_at];
        [first, second]
    }

    fn make_adjoin_graph_vertex_indices(
        indices: &[u32],
        vertices: &[glam::Vec3],
    ) -> Vec<HashSet<GraphVertexIndex>> {
        let mut adjoin_indices: Vec<HashSet<GraphVertexIndex>> = Vec::new();
        adjoin_indices.resize(vertices.len(), HashSet::new());

        for (at, vertex_index) in indices.iter().enumerate() {
            let other_two_vertex_indices = Self::find_other_two_vertex_indices(at, indices);
            adjoin_indices[*vertex_index as usize].extend(other_two_vertex_indices);
        }
        adjoin_indices
    }

    fn make_edges(adjoin_graph_vertex_indices: &[HashSet<GraphVertexIndex>]) -> HashSet<Edge> {
        let mut edges: HashSet<Edge> = HashSet::new();
        for (graph_vertex_index, adjoin_indices) in adjoin_graph_vertex_indices.iter().enumerate() {
            for adjoin_vertex_index in adjoin_indices.clone() {
                let edge = Edge::new(graph_vertex_index as u32, adjoin_vertex_index);
                edges.insert(edge);
            }
        }
        edges
    }

    fn make_graph_vertex_associated_indices(
        indices: &[u32],
        vertices: &[glam::Vec3],
    ) -> Vec<HashSet<usize>> {
        let mut graph_vertex_associated_indices: Vec<HashSet<usize>> = Vec::new();
        graph_vertex_associated_indices.resize(vertices.len(), HashSet::new());
        for (at, vertex_index) in indices.iter().enumerate() {
            graph_vertex_associated_indices[*vertex_index as usize].insert(at);
        }
        graph_vertex_associated_indices
    }

    fn to_graph(indices: &[u32], vertices: &[glam::Vec3]) -> Graph {
        debug_assert_eq!(indices.len() % 3, 0);
        let adjoin_graph_vertex_indices = Self::make_adjoin_graph_vertex_indices(indices, vertices);
        let edges = Self::make_edges(&adjoin_graph_vertex_indices);
        let graph_vertex_associated_indices =
            Self::make_graph_vertex_associated_indices(indices, vertices);

        Graph {
            adjoin_indices: adjoin_graph_vertex_indices,
            graph_vertex_associated_indices,
            edges,
        }
    }

    // fn build_mesh_clusters(
    //     indices: &[u32],
    //     graph: &Graph,
    //     partitions: &Vec<Vec<GraphVertexIndex>>,
    // ) -> Vec<Vec<MeshVertexIndex>> {
    //     let mut cluster_indices: Vec<Vec<MeshVertexIndex>> = Vec::new();
    //     for partition in partitions {
    //         let mut triangles: HashSet<(usize, usize, usize)> = HashSet::new();
    //         for graph_vertex_index in partition {
    //             let associated_indices =
    //                 &graph.graph_vertex_associated_indices[*graph_vertex_index as usize];
    //             for index in associated_indices {
    //                 let triangle = Self::fill_indices_triangle(*index);
    //                 triangles.insert(triangle);
    //             }
    //         }
    //         let sub_indices: Vec<GraphVertexIndex> = triangles
    //             .iter()
    //             .flat_map(|triangle| {
    //                 [
    //                     indices[triangle.0],
    //                     indices[triangle.1],
    //                     indices[triangle.2],
    //                 ]
    //             })
    //             .collect();
    //         cluster_indices.push(sub_indices);
    //     }
    //     cluster_indices
    // }

    fn build_mesh_clusters(
        graph: &Graph,
        partitions: &Vec<Vec<GraphVertexIndex>>,
    ) -> Vec<Vec<usize>> {
        let mut cluster_indices: Vec<Vec<usize>> = Vec::new();
        for partition in partitions {
            let mut triangles: HashSet<usize> = HashSet::new();
            for graph_vertex_index in partition {
                let associated_indices =
                    &graph.graph_vertex_associated_indices[*graph_vertex_index as usize];
                for index in associated_indices {
                    let triangle = *index / 3 * 3;
                    triangles.insert(triangle);
                }
            }
            cluster_indices.push(triangles.iter().map(|x| *x).collect::<Vec<usize>>());
        }
        cluster_indices
    }

    // fn fill_indices_triangle(index: usize) -> (usize, usize, usize) {
    //     let start = index / 3 * 3;
    //     (start, start + 1, start + 2)
    // }

    pub fn partition(
        indices: &[u32],
        vertices: &[glam::Vec3],
        num_parts: u32,
        gpmetis_program_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<Vec<Vec<usize>>> {
        let output_path = std::path::Path::new("./t.graph").to_path_buf();
        let output_path = rs_foundation::absolute_path(output_path)
            .map_err(|err| crate::error::Error::IO(err, None))?;
        let graph = Self::to_graph(indices, vertices);
        Self::write_graph(&graph, output_path.clone())?;
        let partition = Self::internal_partition(gpmetis_program_path, output_path, num_parts)?;
        // let _ = std::fs::remove_file(output_path);

        let mut partition_ret: Vec<Vec<GraphVertexIndex>> = vec![vec![]; num_parts as usize];

        for (graph_vertex_index, which_part) in partition.iter().enumerate() {
            let value = partition_ret
                .get_mut(*which_part as usize)
                .expect("Should not be null");
            value.push(graph_vertex_index as GraphVertexIndex);
        }

        Ok(Self::build_mesh_clusters(&graph, &partition_ret))
    }

    fn internal_partition(
        gpmetis_program_path: impl AsRef<std::path::Path>,
        graph_file_path: impl AsRef<std::path::Path>,
        num_parts: u32,
    ) -> crate::error::Result<Vec<u32>> {
        let mut cmd = Command::new(gpmetis_program_path.as_ref());
        cmd.args([
            graph_file_path
                .as_ref()
                .to_str()
                .ok_or(crate::error::Error::Other(Some(format!(""))))?,
            &num_parts.to_string(),
        ]);
        let output = cmd.output();
        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(crate::error::Error::Other(Some(format!(
                        "{}",
                        String::from_utf8(output.stderr)
                            .map_err(|err| crate::error::Error::FromUtf8Error(err))?
                    ))));
                }
            }
            Err(err) => {
                return Err(crate::error::Error::IO(err, None));
            }
        }

        let file_name = graph_file_path
            .as_ref()
            .file_name()
            .map(|x| x.to_str().map(|x| format!("{x}.part.{}", num_parts)))
            .flatten()
            .ok_or(crate::error::Error::Other(Some(format!("No parent"))))?;

        let graph_partition_file_path = graph_file_path.as_ref().with_file_name(file_name);
        let file = std::fs::File::open(graph_partition_file_path.clone()).map_err(|err| {
            crate::error::Error::IO(err, Some(format!("{:?}", graph_partition_file_path)))
        })?;
        let reader = std::io::BufReader::new(file);
        let mut partition: Vec<u32> = Vec::new();
        for line in std::io::BufRead::lines(reader) {
            let which_part: u32 = line
                .map_err(|err| crate::error::Error::IO(err, None))?
                .trim()
                .parse()
                .map_err(|err| crate::error::Error::ParseIntError(err))?;
            partition.push(which_part);
        }
        // let _ = std::fs::remove_file(graph_partition_file_path);
        Ok(partition)
    }

    fn write_graph(
        graph: &Graph,
        output_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<()> {
        let mut content = String::new();
        content.push_str(&format!(
            "{} {}\n",
            graph.get_num_vertices(),
            graph.get_num_edges()
        ));
        for indices in &graph.adjoin_indices {
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

#[cfg(test)]
mod tests {
    use crate::bindings::*;
    use crate::metis::*;

    #[test]
    fn test_case() {
        let graph_t = unsafe { ::std::mem::zeroed::<graph_t>() };
    }
}
