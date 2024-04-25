use crate::asset::Asset;
use crate::resource_type::EResourceType;
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkinMesh {
    pub name: String,
    pub url: url::Url,
    pub vertexes: Vec<SkinMeshVertex>,
    pub indexes: Vec<u32>,
    pub bone_paths: Vec<String>,
}

impl Asset for SkinMesh {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::SkinMesh
    }
}
