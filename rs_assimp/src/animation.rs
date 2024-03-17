use std::marker::PhantomData;
use crate::{mesh_anim::MeshAnim, mesh_morph_anim::MeshMorphAnim, node_anim::NodeAnim};

pub struct Animation<'a> {
    c: &'a mut russimp_sys::aiAnimation,
    pub name: String,
    pub duration: f64,
    pub ticks_per_second: f64,
    pub channels: Vec<NodeAnim<'a>>,
    pub mesh_channels: Vec<MeshAnim<'a>>,
    pub morph_mesh_channels: Vec<MeshMorphAnim<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Animation<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiAnimation) -> Animation<'a> {
        let name = c.mName.into();
        let duration = c.mDuration;
        let ticks_per_second = c.mTicksPerSecond;
        let mut channels: Vec<NodeAnim<'a>> = vec![];
        let mut mesh_channels: Vec<MeshAnim<'a>> = vec![];
        let mut morph_mesh_channels: Vec<MeshMorphAnim<'a>> = vec![];
        if !c.mChannels.is_null() {
            let ai_channels =
                unsafe { std::slice::from_raw_parts_mut(c.mChannels, c.mNumChannels as _) };
            for item in ai_channels.iter_mut() {
                channels.push(NodeAnim::borrow_from(unsafe { item.as_mut().unwrap() }));
            }
        }
        if !c.mMeshChannels.is_null() {
            let ai_channels =
                unsafe { std::slice::from_raw_parts_mut(c.mMeshChannels, c.mNumMeshChannels as _) };
            for item in ai_channels.iter_mut() {
                mesh_channels.push(MeshAnim::borrow_from(unsafe { item.as_mut().unwrap() }));
            }
        }
        if !c.mMorphMeshChannels.is_null() {
            let ai_channels = unsafe {
                std::slice::from_raw_parts_mut(c.mMorphMeshChannels, c.mNumMorphMeshChannels as _)
            };
            for item in ai_channels.iter_mut() {
                morph_mesh_channels.push(MeshMorphAnim::borrow_from(unsafe {
                    item.as_mut().unwrap()
                }));
            }
        }
        Animation {
            c,
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
