use super::{
    level::Level, skeleton::Skeleton, skeleton_animation::SkeletonAnimation,
    skeleton_mesh::SkeletonMesh, static_mesh::StaticMesh, texture::TextureFile,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EContentFileType {
    StaticMesh(Rc<RefCell<StaticMesh>>),
    SkeletonMesh(Rc<RefCell<SkeletonMesh>>),
    SkeletonAnimation(Rc<RefCell<SkeletonAnimation>>),
    Skeleton(Rc<RefCell<Skeleton>>),
    Texture(Rc<RefCell<TextureFile>>),
    Level(Rc<RefCell<Level>>),
}
