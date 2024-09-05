use super::{
    ibl::IBL, level::Level, material::Material, particle_system::ParticleSystem,
    skeleton::Skeleton, skeleton_animation::SkeletonAnimation, skeleton_mesh::SkeletonMesh,
    sound::Sound, static_mesh::StaticMesh, texture::TextureFile,
};
use rs_artifact::asset::Asset;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Serialize, Deserialize, Clone)]
pub enum EContentFileType {
    StaticMesh(Rc<RefCell<StaticMesh>>),
    SkeletonMesh(Rc<RefCell<SkeletonMesh>>),
    SkeletonAnimation(Rc<RefCell<SkeletonAnimation>>),
    Skeleton(Rc<RefCell<Skeleton>>),
    Texture(Rc<RefCell<TextureFile>>),
    Level(Rc<RefCell<Level>>),
    Material(Rc<RefCell<Material>>),
    IBL(Rc<RefCell<IBL>>),
    ParticleSystem(Rc<RefCell<ParticleSystem>>),
    Sound(Rc<RefCell<Sound>>),
}

impl EContentFileType {
    pub fn get_type_text(&self) -> String {
        match self {
            EContentFileType::StaticMesh(_) => "StaticMesh".to_string(),
            EContentFileType::SkeletonMesh(_) => "SkeletonMesh".to_string(),
            EContentFileType::SkeletonAnimation(_) => "SkeletonAnimation".to_string(),
            EContentFileType::Skeleton(_) => "Skeleton".to_string(),
            EContentFileType::Texture(_) => "Texture".to_string(),
            EContentFileType::Level(_) => "Level".to_string(),
            EContentFileType::Material(_) => "Material".to_string(),
            EContentFileType::IBL(_) => "IBL".to_string(),
            EContentFileType::ParticleSystem(_) => "ParticleSystem".to_string(),
            EContentFileType::Sound(_) => "Sound".to_string(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            EContentFileType::StaticMesh(content) => content.borrow().get_name(),
            EContentFileType::SkeletonMesh(content) => content.borrow().get_name(),
            EContentFileType::SkeletonAnimation(content) => content.borrow().get_name(),
            EContentFileType::Skeleton(content) => content.borrow().get_name(),
            EContentFileType::Texture(content) => content.borrow().get_name(),
            EContentFileType::Level(content) => content.borrow().get_name(),
            EContentFileType::Material(content) => content.borrow().get_name(),
            EContentFileType::IBL(content) => content.borrow().get_name(),
            EContentFileType::ParticleSystem(content) => content.borrow().get_name(),
            EContentFileType::Sound(content) => content.borrow().get_name(),
        }
    }

    pub fn get_url(&self) -> url::Url {
        match self {
            EContentFileType::StaticMesh(content) => content.borrow().get_url(),
            EContentFileType::SkeletonMesh(content) => content.borrow().get_url(),
            EContentFileType::SkeletonAnimation(content) => content.borrow().get_url(),
            EContentFileType::Skeleton(content) => content.borrow().get_url(),
            EContentFileType::Texture(content) => content.borrow().get_url(),
            EContentFileType::Level(content) => content.borrow().get_url(),
            EContentFileType::Material(content) => content.borrow().get_url(),
            EContentFileType::IBL(content) => content.borrow().get_url(),
            EContentFileType::ParticleSystem(content) => content.borrow().get_url(),
            EContentFileType::Sound(content) => content.borrow().get_url(),
        }
    }
}
