use crate::mesh_morph_key::MeshMorphKey;
use std::marker::PhantomData;

pub struct MeshMorphAnim<'a> {
    c: &'a mut russimp_sys::aiMeshMorphAnim,
    pub name: String,
    pub keys: Vec<MeshMorphKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshMorphAnim<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMeshMorphAnim) -> MeshMorphAnim<'a> {
        let name = c.mName.into();
        let keys = unsafe { std::slice::from_raw_parts_mut(c.mKeys, c.mNumKeys as _) };
        let keys = keys
            .iter_mut()
            .map(|x| MeshMorphKey::borrow_from(x))
            .collect();
        MeshMorphAnim {
            c,
            name,
            keys,
            marker: PhantomData,
        }
    }
}
