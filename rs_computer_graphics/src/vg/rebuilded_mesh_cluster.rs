use crate::{brigde_data::mesh_vertex::MeshVertex, glam_color};

pub struct RebuildedMeshCluster {
    pub indices: Vec<u32>,
    pub vertices: Vec<MeshVertex>,
    pub vertex_colors: Vec<glam::Vec4>,
    pub vertex_positions: Vec<glam::Vec3>,
}

impl RebuildedMeshCluster {
    pub fn rebuild(sub_indices: &[u32], vertices: &[MeshVertex]) -> RebuildedMeshCluster {
        let vertices: Vec<MeshVertex> = sub_indices.iter().map(|x| vertices[*x as usize]).collect();
        let indices: Vec<u32> = (0..sub_indices.len()).map(|x| x as u32).collect();
        let mut vertex_colors: Vec<glam::Vec4> = Vec::new();
        let mut vertex_positions: Vec<glam::Vec3> = Vec::new();
        let vertex_color = glam_color::random();
        for vertex_index in indices.iter() {
            let mesh_vertex = vertices.get(*vertex_index as usize).unwrap();
            vertex_positions.push(mesh_vertex.position);
            vertex_colors.push(vertex_color);
        }
        RebuildedMeshCluster {
            indices,
            vertices,
            vertex_colors,
            vertex_positions,
        }
    }
}
