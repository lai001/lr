use crate::{graph::TriangleGraph, vertex_position::VertexPosition};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::NonZero,
    sync::{Arc, Mutex},
};

const MAX_TRIANGLES_LEN: usize = 128;

struct TaskInput {
    graph: Arc<TriangleGraph>,
    inc_id: Arc<Mutex<i32>>,
    depth: u32,
}

struct PartitionOutput {
    parent: i32,
    cluster_id: i32,
    depth: u32,
    graph: Arc<TriangleGraph>,
    occluder_indices: Vec<u32>,
    aabb: rapier3d::prelude::Aabb,
}

struct TaskOutput {
    partition_outputs: crate::error::Result<Vec<PartitionOutput>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cluster {
    pub id: i32,
    pub lod: u32,
    pub depth: u32,
    pub indices: Vec<u32>,
    pub occluder_indices: Vec<u32>,
    pub parent: Option<i32>,
    pub childs: Vec<i32>,
    pub aabb: rapier3d::prelude::Aabb,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClusterCollection {
    pub clusters: HashMap<i32, Cluster>,
    pub root_id: i32,
    pub max_lod: u32,
}

impl ClusterCollection {
    pub fn get_leaf_cluster_ids(&self) -> Vec<i32> {
        let mut clusters: Vec<i32> = Vec::new();
        for (id, cluster) in self.clusters.iter() {
            if cluster.childs.is_empty() {
                clusters.push(*id);
            }
        }
        clusters
    }

    pub fn get_leaf_clusters(&self) -> Vec<&Cluster> {
        let mut clusters: Vec<&Cluster> = Vec::new();
        for cluster in self.clusters.values() {
            if cluster.childs.is_empty() {
                clusters.push(cluster);
            }
        }
        clusters
    }

    pub fn from_indexed_vertices(
        indices: &[u32],
        vertices: &[VertexPosition],
        gpmetis_program_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<ClusterCollection> {
        let mut collection = HashMap::new();
        let mut inc_id: i32 = 0;
        let mut max_depth: u32 = 0;
        let childs = Self::partition_from_indexed_vertices(
            indices,
            vertices,
            gpmetis_program_path,
            Some(0),
            &mut inc_id,
            &mut collection,
            1,
            &mut max_depth,
        );
        let root_aabb = indexed_vertices_to_aabb(
            indices,
            &vertices.iter().map(|x| x.p).collect::<Vec<glam::Vec3>>(),
        );
        let root_cluster = Cluster {
            id: 0,
            lod: 0,
            indices: indices.to_vec(),
            parent: None,
            childs,
            depth: 0,
            aabb: root_aabb,
            occluder_indices: simplify_mesh(indices, vertices),
        };
        collection.insert(0, root_cluster);
        Self::resolve_lod(&mut collection, max_depth);
        Self::fill(&mut collection, max_depth, inc_id);
        Ok(ClusterCollection {
            clusters: collection,
            root_id: 0,
            max_lod: max_depth,
        })
    }

    pub fn parallel_from_indexed_vertices(
        indices: &[u32],
        vertices: Arc<Vec<VertexPosition>>,
        gpmetis_program_path: Option<std::path::PathBuf>,
    ) -> crate::error::Result<ClusterCollection> {
        let mut collection = HashMap::new();
        let mut inc_id: i32 = 0;
        let mut max_depth: u32 = 0;
        let childs = Self::parallel_partition_from_indexed_vertices(
            indices,
            vertices.clone(),
            gpmetis_program_path,
            Some(0),
            &mut inc_id,
            &mut collection,
            1,
            &mut max_depth,
        );
        let root_aabb = indexed_vertices_to_aabb(
            indices,
            &vertices.iter().map(|x| x.p).collect::<Vec<glam::Vec3>>(),
        );
        let root_cluster = Cluster {
            id: 0,
            lod: 0,
            indices: indices.to_vec(),
            parent: None,
            childs,
            depth: 0,
            aabb: root_aabb,
            occluder_indices: simplify_mesh(indices, &vertices),
        };
        collection.insert(0, root_cluster);
        Self::resolve_lod(&mut collection, max_depth);
        Self::fill(&mut collection, max_depth, inc_id);
        Ok(ClusterCollection {
            clusters: collection,
            root_id: 0,
            max_lod: max_depth,
        })
    }

    pub fn parallel_from_indexed_vertices2(
        indices: Arc<Vec<u32>>,
        vertices: Arc<Vec<VertexPosition>>,
    ) -> crate::error::Result<ClusterCollection> {
        const ROOT_ID: i32 = 0;
        let mut collection = HashMap::new();
        let inc_id = Arc::new(Mutex::new(ROOT_ID));
        let mut max_depth: u32 = 1;
        let mut running_tasks = 0;
        let (sender, receiver) = std::sync::mpsc::channel::<TaskOutput>();
        let input = TaskInput {
            inc_id: inc_id.clone(),
            depth: max_depth,
            graph: Arc::new(TriangleGraph::parallel_from_indexed_vertices(
                &indices,
                vertices.clone(),
            )),
        };

        running_tasks += 1;
        Self::partition_from_indexed_vertices_background(
            ROOT_ID,
            input,
            vertices.clone(),
            sender.clone(),
        );

        let mut all_partition_outputs: HashMap<u32, Vec<PartitionOutput>> = HashMap::new();
        while let Ok(output) = receiver.recv() {
            running_tasks -= 1;
            match output.partition_outputs {
                Ok(mut partition_outputs) => {
                    #[cfg(debug_assertions)]
                    {
                        let s: std::collections::HashSet<u32> =
                            partition_outputs.iter().map(|x| x.depth).collect();
                        assert_eq!(s.len(), 1);
                    }
                    for output in partition_outputs.iter() {
                        if (output.graph.get_triangles().len()) > MAX_TRIANGLES_LEN {
                            let input = TaskInput {
                                inc_id: inc_id.clone(),
                                depth: output.depth + 1,
                                graph: output.graph.clone(),
                            };
                            running_tasks += 1;
                            Self::partition_from_indexed_vertices_background(
                                output.cluster_id,
                                input,
                                vertices.clone(),
                                sender.clone(),
                            );
                        }
                    }
                    if let Some(first) = partition_outputs.first() {
                        all_partition_outputs
                            .entry(first.depth)
                            .or_default()
                            .append(&mut partition_outputs);
                    }
                }
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
            if running_tasks == 0 {
                break;
            }
        }

        let root_aabb = indexed_vertices_to_aabb(
            &indices,
            &vertices.iter().map(|x| x.p).collect::<Vec<glam::Vec3>>(),
        );
        let root_cluster = Cluster {
            id: ROOT_ID,
            lod: 0,
            depth: 0,
            indices: indices.to_vec(),
            occluder_indices: simplify_mesh(&indices, &vertices),
            parent: None,
            childs: vec![],
            aabb: root_aabb,
        };
        collection.insert(ROOT_ID, root_cluster);

        let mut keys = all_partition_outputs.keys().cloned().collect::<Vec<u32>>();
        max_depth = keys.len() as u32;
        keys.sort_by(|l, r| l.cmp(r));
        for depth in keys {
            let value = all_partition_outputs
                .remove(&depth)
                .ok_or(crate::error::Error::Other(None))?;
            for partition_output in value {
                let triangles = partition_output.graph.get_triangles();
                let mut indices: Vec<u32> = Vec::with_capacity(triangles.len() * 3);

                for triangle in triangles {
                    indices.append(&mut triangle.get_indices().to_vec());
                }

                // while Arc::strong_count(&partition_output.indices) != 1 {}
                // let indices = Arc::try_unwrap(partition_output.indices).unwrap();
                let cluster = Cluster {
                    id: partition_output.cluster_id,
                    lod: 0,
                    depth: partition_output.depth,
                    indices,
                    occluder_indices: partition_output.occluder_indices,
                    parent: Some(partition_output.parent),
                    childs: vec![],
                    aabb: partition_output.aabb,
                };
                collection.insert(partition_output.cluster_id, cluster);
            }
        }

        let mut resolve: HashMap<i32, Vec<i32>> = HashMap::new();
        for (k, v) in collection.iter_mut() {
            if let Some(parent) = v.parent {
                let value = resolve.entry(parent).or_default();
                value.push(*k);
            }
        }
        for (k, mut v) in resolve {
            if let Some(cluster) = collection.get_mut(&k) {
                cluster.childs.append(&mut v);
            }
        }

        Self::resolve_lod(&mut collection, max_depth);
        let id = { *inc_id.lock().unwrap() };
        Self::fill(&mut collection, max_depth, id);
        Ok(ClusterCollection {
            clusters: collection,
            root_id: ROOT_ID,
            max_lod: max_depth,
        })
    }

    fn resolve_lod(clusters: &mut HashMap<i32, Cluster>, max_depth: u32) {
        for v in clusters.values_mut() {
            v.lod = max_depth - v.depth;
        }
    }

    fn fill(clusters: &mut HashMap<i32, Cluster>, max_depth: u32, mut inc_id: i32) {
        let mut collection: Vec<Cluster> = vec![];
        for cluster in clusters.values() {
            if cluster.depth == max_depth - 1 {
                if cluster.childs.is_empty() {
                    inc_id = inc_id + 1;
                    let id = inc_id;
                    let mut new_cluster = cluster.clone();
                    new_cluster.id = id;
                    new_cluster.depth = max_depth;
                    new_cluster.lod = 0;
                    new_cluster.parent = Some(cluster.id);
                    collection.push(new_cluster);
                }
            }
        }

        for new_cluster in collection {
            clusters.insert(new_cluster.id, new_cluster);
        }
    }

    pub fn partition_from_indexed_vertices(
        indices: &[u32],
        vertices: &[VertexPosition],
        gpmetis_program_path: impl AsRef<std::path::Path>,
        parent: Option<i32>,
        inc_id: &mut i32,
        collection: &mut HashMap<i32, Cluster>,
        current_depth: u32,
        max_depth: &mut u32,
    ) -> Vec<i32> {
        let partitions = crate::metis::Metis::partition_from_indexed_vertices(indices, vertices, 2);
        let partitions = match partitions {
            Ok(partitions) => partitions,
            Err(err) => {
                log::warn!("{}", err);
                return vec![];
            }
        };

        let mut clusters = vec![];

        *max_depth = (*max_depth).max(current_depth);

        for partition in partitions {
            *inc_id = *inc_id + 1;

            let id = *inc_id;
            let mut childs = vec![];
            if (partition.len() / 3) > MAX_TRIANGLES_LEN {
                childs = Self::partition_from_indexed_vertices(
                    &partition,
                    vertices,
                    gpmetis_program_path.as_ref(),
                    Some(id),
                    inc_id,
                    collection,
                    current_depth + 1,
                    max_depth,
                );
            }
            let aabb = indexed_vertices_to_aabb(
                &partition,
                &vertices.iter().map(|x| x.p).collect::<Vec<glam::Vec3>>(),
            );
            let optimized_indices = simplify_mesh(&partition, &vertices);
            let cluster = Cluster {
                id,
                lod: 0,
                indices: partition,
                parent,
                childs,
                depth: current_depth,
                aabb,
                occluder_indices: optimized_indices,
            };
            collection.insert(id, cluster);
            clusters.push(id);
        }

        clusters
    }

    pub fn parallel_partition_from_indexed_vertices(
        indices: &[u32],
        vertices: Arc<Vec<VertexPosition>>,
        gpmetis_program_path: Option<std::path::PathBuf>,
        parent: Option<i32>,
        inc_id: &mut i32,
        collection: &mut HashMap<i32, Cluster>,
        current_depth: u32,
        max_depth: &mut u32,
    ) -> Vec<i32> {
        let partitions = crate::metis::Metis::parallel_partition_from_indexed_vertices(
            indices,
            vertices.clone(),
            2,
        );
        let partitions = match partitions {
            Ok(partitions) => partitions,
            Err(err) => {
                log::warn!("{}", err);
                return vec![];
            }
        };

        let mut clusters = vec![];

        *max_depth = (*max_depth).max(current_depth);

        for partition in partitions {
            *inc_id = *inc_id + 1;

            let id = *inc_id;
            let mut childs = vec![];
            if (partition.len() / 3) > MAX_TRIANGLES_LEN {
                childs = Self::parallel_partition_from_indexed_vertices(
                    &partition,
                    vertices.clone(),
                    gpmetis_program_path.clone(),
                    Some(id),
                    inc_id,
                    collection,
                    current_depth + 1,
                    max_depth,
                );
            }
            let aabb = indexed_vertices_to_aabb(
                &partition,
                &vertices.iter().map(|x| x.p).collect::<Vec<glam::Vec3>>(),
            );
            let optimized_indices = simplify_mesh(&partition, &vertices);
            let cluster = Cluster {
                id,
                lod: 0,
                indices: partition,
                parent,
                childs,
                depth: current_depth,
                aabb,
                occluder_indices: optimized_indices,
            };
            collection.insert(id, cluster);
            clusters.push(id);
        }

        clusters
    }

    pub fn plat(&self) -> Vec<Vec<Cluster>> {
        let mut clusters = Vec::with_capacity(self.max_lod as usize + 1);
        for lod in 0..(self.max_lod + 1) {
            let mut sub = vec![];

            for v in self.clusters.values() {
                if v.lod == lod {
                    sub.push(v.clone());
                }
            }
            assert_eq!(sub.is_empty(), false);
            clusters.push(sub);
        }
        return clusters;
    }

    pub fn get_root_aabb(&self) -> Option<&rapier3d::prelude::Aabb> {
        if let Some(root) = self.clusters.get(&self.root_id) {
            return Some(&root.aabb);
        }
        return None;
    }

    fn partition_from_indexed_vertices_background(
        parent: i32,
        input: TaskInput,
        vertices: Arc<Vec<VertexPosition>>,
        sender: std::sync::mpsc::Sender<TaskOutput>,
    ) {
        rs_core_minimal::thread_pool::ThreadPool::global().spawn(move || {
            let partitions =
                crate::metis::Metis::partition_from_graph(&input.graph, NonZero::new(2).unwrap());
            let mut partitions = match partitions {
                Ok(partitions) => partitions,
                Err(err) => {
                    let _ = sender.send(TaskOutput {
                        partition_outputs: Err(err),
                    });
                    return;
                }
            };
            let mut outputs = Vec::with_capacity(partitions.len());

            let base_id: i32 = {
                match input.inc_id.lock() {
                    Ok(mut id) => {
                        let base_id = *id + 1;
                        *id += partitions.len() as i32;
                        base_id
                    }
                    Err(err) => {
                        panic!("{}", err);
                    }
                }
            };

            for (i, graph) in partitions.drain(..).enumerate() {
                let mut partition = Vec::<u32>::with_capacity(graph.get_triangles().len() * 3);
                for triangle in graph.get_triangles() {
                    let mut other = triangle.get_indices().to_vec();
                    partition.append(&mut other);
                }
                let aabb = indexed_vertices_to_aabb2(&partition, vertices.clone());
                let optimized_indices = simplify_mesh(&partition, &vertices);

                let task_output = PartitionOutput {
                    depth: input.depth,
                    occluder_indices: optimized_indices,
                    aabb,
                    cluster_id: base_id + i as i32,
                    parent,
                    graph: Arc::new(graph),
                };
                outputs.push(task_output);
            }
            let _ = sender.send(TaskOutput {
                partition_outputs: Ok(outputs),
            });
        });
    }
}

fn indexed_vertices_to_aabb(indices: &[u32], vertices: &[glam::Vec3]) -> rapier3d::prelude::Aabb {
    let _ = tracy_client::span!();
    let points: Vec<glam::Vec3> = indices.iter().map(|x| vertices[*x as usize]).collect();

    let aabb = rapier3d::prelude::Aabb::from_points(points);
    aabb
}

fn indexed_vertices_to_aabb2(
    indices: &[u32],
    vertices: Arc<Vec<VertexPosition>>,
) -> rapier3d::prelude::Aabb {
    let _ = tracy_client::span!();
    let points: Vec<glam::Vec3> = indices.iter().map(|x| vertices[*x as usize].p).collect();

    let aabb = rapier3d::prelude::Aabb::from_points(points);
    aabb
}

fn get_vertex_adapter<'a>(vertices: &'a [VertexPosition]) -> meshopt::VertexDataAdapter<'a> {
    let position_offset = std::mem::offset_of!(VertexPosition, p);
    let vertex_stride = std::mem::size_of::<VertexPosition>();
    let vertex_data = meshopt::typed_to_bytes(&vertices);
    meshopt::VertexDataAdapter::new(vertex_data, vertex_stride, position_offset)
        .expect("Create a valid vertex data reader")
}

fn simplify_mesh(indices: &[u32], vertices: &[VertexPosition]) -> Vec<u32> {
    let _ = tracy_client::span!();
    let vertex_adapter = get_vertex_adapter(vertices);
    let threshold = 0.7f32.powf(1.0);
    let target_index_count = (indices.len() as f32 * threshold) as usize / 3 * 3;
    let target_error = 1e-3f32;
    let lod = meshopt::simplify(
        indices,
        &vertex_adapter,
        std::cmp::min(indices.len(), target_index_count),
        target_error,
        meshopt::SimplifyOptions::LockBorder,
        None,
    );
    lod
}
