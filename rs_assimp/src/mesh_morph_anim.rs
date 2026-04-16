use crate::{convert::ConvertToString, mesh_morph_key::MeshMorphKey};
use rs_assimp_sys::*;
use std::marker::PhantomData;

pub struct MeshMorphAnim<'a> {
    _ai_mesh_morph_anim: &'a mut aiMeshMorphAnim,
    pub name: String,
    pub keys: Vec<MeshMorphKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshMorphAnim<'a> {
    pub fn borrow_from(ai_mesh_morph_anim: &'a mut aiMeshMorphAnim) -> MeshMorphAnim<'a> {
        let name = ai_mesh_morph_anim.mName.to_string();
        let keys = unsafe {
            std::slice::from_raw_parts_mut(
                ai_mesh_morph_anim.mKeys,
                ai_mesh_morph_anim.mNumKeys as _,
            )
        };
        let keys = keys
            .iter_mut()
            .map(|x| MeshMorphKey::borrow_from(x))
            .collect();
        MeshMorphAnim {
            _ai_mesh_morph_anim: ai_mesh_morph_anim,
            name,
            keys,
            marker: PhantomData,
        }
    }
}
