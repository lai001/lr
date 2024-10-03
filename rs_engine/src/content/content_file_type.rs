use super::{
    curve::Curve, ibl::IBL, level::Level, material::Material, particle_system::ParticleSystem,
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
    Curve(Rc<RefCell<Curve>>),
}

macro_rules! common_fn {
    ($($x:tt),*) => {
        pub fn get_type_text(&self) -> String {
            match self {
                $(EContentFileType::$x(_) => stringify!($x).to_string(),)*
            }
        }

        pub fn get_name(&self) -> String {
            match self {
                $(EContentFileType::$x(content) => content.borrow().get_name(),)*
            }
        }

        pub fn get_url(&self) -> url::Url {
            match self {
                $(EContentFileType::$x(content) => content.borrow().get_url(),)*
            }
        }
    };
}

impl EContentFileType {
    common_fn!(
        StaticMesh,
        SkeletonMesh,
        SkeletonAnimation,
        Skeleton,
        Texture,
        Level,
        Material,
        IBL,
        ParticleSystem,
        Sound,
        Curve
    );
}
