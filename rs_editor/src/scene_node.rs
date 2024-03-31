use crate::skeleton_mesh::SkeletonMesh;
use rs_engine::resource_manager::ResourceManager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneComponent {
    pub name: String,
    pub transformation: glam::Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticMeshComponent {
    pub name: String,
    pub static_mesh: Option<url::Url>,
    pub transformation: glam::Mat4,
}

#[derive(Debug, Clone)]
pub struct SkeletonMeshTreeNode {
    pub transformatin: glam::Mat4,
    pub skeleton_mesh: Option<SkeletonMesh>,
    pub childs: Vec<SkeletonMeshTreeNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkeletonMeshComponent {
    pub name: String,
    pub skeleton: Option<url::Url>,
    pub skeleton_meshes: Vec<url::Url>,
    pub animation: Option<url::Url>,
    #[serde(skip)]
    pub skeleton_mesh_tree: Option<SkeletonMeshTreeNode>,
    pub transformation: glam::Mat4,
}

impl SkeletonMeshComponent {
    pub fn initialize(&mut self, resource_manager: ResourceManager) {}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EComponentType {
    SceneComponent(SceneComponent),
    StaticMeshComponent(StaticMeshComponent),
    SkeletonMeshComponent(SkeletonMeshComponent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneNode {
    pub component: EComponentType,
    pub childs: Vec<SceneNode>,
}
