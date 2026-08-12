use crate::mesh_vertex::MeshVertex;
use serde::Deserialize;
use serde::Serialize;

#[repr(C)]
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SkinMeshVertex {
    pub vertex_color: glam::Vec4,
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub tangent: glam::Vec3,
    pub bitangent: glam::Vec3,
    pub tex_coord: glam::Vec2,
    pub bones: [i32; 4],
    pub weights: [f32; 4],
}

impl SkinMeshVertex {
    pub fn to_mesh_vertex(&self) -> MeshVertex {
        MeshVertex {
            vertex_color: self.vertex_color,
            position: self.position,
            normal: self.normal,
            tangent: self.tangent,
            bitangent: self.bitangent,
            tex_coord: self.tex_coord,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkinMesh {
    pub name: String,
    pub url: url::Url,
    pub vertexes: Vec<SkinMeshVertex>,
    pub indexes: Vec<u32>,
    pub bone_paths: Vec<String>,
}

crate::impl_asset!(SkinMesh);
