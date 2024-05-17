use crate::mesh_morph_key::MeshMorphKey;
use std::marker::PhantomData;

pub struct MeshMorphAnim<'a> {
    _ai_mesh_morph_anim: &'a mut russimp_sys::aiMeshMorphAnim,
    pub name: String,
    pub keys: Vec<MeshMorphKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshMorphAnim<'a> {
    pub fn borrow_from(
        ai_mesh_morph_anim: &'a mut russimp_sys::aiMeshMorphAnim,
    ) -> MeshMorphAnim<'a> {
        let name = ai_mesh_morph_anim.mName.into();
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
