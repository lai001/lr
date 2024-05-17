use crate::{mesh_anim::MeshAnim, mesh_morph_anim::MeshMorphAnim, node::Node, node_anim::NodeAnim};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct Animation<'a> {
    _ai_animation: &'a mut russimp_sys::aiAnimation,
    pub name: String,
    pub duration: f64,
    pub ticks_per_second: f64,
    pub channels: Vec<NodeAnim<'a>>,
    pub mesh_channels: Vec<MeshAnim<'a>>,
    pub morph_mesh_channels: Vec<MeshMorphAnim<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Animation<'a> {
    pub fn borrow_from(
        ai_animation: &'a mut russimp_sys::aiAnimation,
        map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> Animation<'a> {
        let name = ai_animation.mName.into();
        let duration = ai_animation.mDuration;
        let ticks_per_second = ai_animation.mTicksPerSecond;
        let mut channels: Vec<NodeAnim<'a>> = vec![];
        let mut mesh_channels: Vec<MeshAnim<'a>> = vec![];
        let mut morph_mesh_channels: Vec<MeshMorphAnim<'a>> = vec![];
        if !ai_animation.mChannels.is_null() {
            let ai_channels = unsafe {
                std::slice::from_raw_parts_mut(
                    ai_animation.mChannels,
                    ai_animation.mNumChannels as _,
                )
            };
            for item in ai_channels.iter_mut() {
                channels.push(NodeAnim::borrow_from(
                    unsafe { item.as_mut().unwrap() },
                    map,
                ));
            }
        }
        if !ai_animation.mMeshChannels.is_null() {
            let ai_channels = unsafe {
                std::slice::from_raw_parts_mut(
                    ai_animation.mMeshChannels,
                    ai_animation.mNumMeshChannels as _,
                )
            };
            for item in ai_channels.iter_mut() {
                mesh_channels.push(MeshAnim::borrow_from(unsafe { item.as_mut().unwrap() }));
            }
        }
        if !ai_animation.mMorphMeshChannels.is_null() {
            let ai_channels = unsafe {
                std::slice::from_raw_parts_mut(
                    ai_animation.mMorphMeshChannels,
                    ai_animation.mNumMorphMeshChannels as _,
                )
            };
            for item in ai_channels.iter_mut() {
                morph_mesh_channels.push(MeshMorphAnim::borrow_from(unsafe {
                    item.as_mut().unwrap()
                }));
            }
        }
        Animation {
            _ai_animation: ai_animation,
            name,
            duration,
            ticks_per_second,
            channels,
            marker: PhantomData,
            mesh_channels,
            morph_mesh_channels,
        }
    }
}
