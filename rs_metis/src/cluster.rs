use crate::vertex_position::VertexPosition;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

const MAX_TRIANGLES_LEN: usize = 128;

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
        Self::fill(&mut collection, max_depth, &mut inc_id);
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
        Self::fill(&mut collection, max_depth, &mut inc_id);
        Ok(ClusterCollection {
            clusters: collection,
            root_id: 0,
            max_lod: max_depth,
        })
    }

    fn resolve_lod(clusters: &mut HashMap<i32, Cluster>, max_depth: u32) {
        for v in clusters.values_mut() {
            v.lod = max_depth - v.depth;
        }
    }

    fn fill(clusters: &mut HashMap<i32, Cluster>, max_depth: u32, inc_id: &mut i32) {
        let mut collection: Vec<Cluster> = vec![];
        for cluster in clusters.values() {
            if cluster.depth == max_depth - 1 {
                if cluster.childs.is_empty() {
                    *inc_id = *inc_id + 1;
                    let id = *inc_id;
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
        let partitions = crate::metis::Metis::partition_from_indexed_vertices(
            indices,
            vertices,
            2,
            gpmetis_program_path.as_ref(),
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
            gpmetis_program_path.as_ref(),
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
}

fn indexed_vertices_to_aabb(indices: &[u32], vertices: &[glam::Vec3]) -> rapier3d::prelude::Aabb {
    let points: Vec<glam::Vec3> = indices.iter().map(|x| vertices[*x as usize]).collect();

    let points: Vec<rapier3d::math::Point<f32>> = points
        .iter()
        .map(|x| rapier3d::math::Point::<f32>::from_slice(&x.to_array()))
        .collect();

    let aabb = rapier3d::prelude::Aabb::from_points(&points);
    aabb
}

fn get_vertex_adapter(vertices: &[VertexPosition]) -> meshopt::VertexDataAdapter {
    let position_offset = std::mem::offset_of!(VertexPosition, p);
    let vertex_stride = std::mem::size_of::<VertexPosition>();
    let vertex_data = meshopt::typed_to_bytes(&vertices);
    meshopt::VertexDataAdapter::new(vertex_data, vertex_stride, position_offset)
        .expect("Create a valid vertex data reader")
}

fn simplify_mesh(indices: &[u32], vertices: &[VertexPosition]) -> Vec<u32> {
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
