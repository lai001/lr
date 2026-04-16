use crate::{convert::ConvertToString, mesh_key::MeshKey};
use rs_assimp_sys::*;
use std::marker::PhantomData;

pub struct MeshAnim<'a> {
    _ai_mesh_anim: &'a mut aiMeshAnim,
    pub name: String,
    pub mesh_keys: Vec<MeshKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshAnim<'a> {
    pub fn borrow_from(ai_mesh_anim: &'a mut aiMeshAnim) -> MeshAnim<'a> {
        let name = ai_mesh_anim.mName.to_string();
        let mesh_keys = unsafe {
            std::slice::from_raw_parts_mut(ai_mesh_anim.mKeys, ai_mesh_anim.mNumKeys as _)
        };
        let mesh_keys = mesh_keys
            .iter_mut()
            .map(|x| MeshKey::borrow_from(x))
            .collect();
        MeshAnim {
            _ai_mesh_anim: ai_mesh_anim,
            name,
            mesh_keys,
            marker: PhantomData,
        }
    }
}
