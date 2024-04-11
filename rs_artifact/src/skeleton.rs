use crate::asset::Asset;
use crate::resource_type::EResourceType;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VertexWeight {
    pub vertex_id: u32,
    pub weight: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkeletonBone {
    pub path: String,
    pub parent: Option<String>,
    pub childs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkeletonMeshHierarchyNode {
    pub path: String,
    pub transformation: glam::Mat4,
    pub parent: Option<String>,
    pub childs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Skeleton {
    pub name: String,
    pub url: url::Url,
    pub root_bone: String,
    pub root_node: String,
    pub bones: HashMap<String, SkeletonBone>,
    pub skeleton_mesh_hierarchy: HashMap<String, SkeletonMeshHierarchyNode>,
}

impl Asset for Skeleton {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Skeleton
    }
}
