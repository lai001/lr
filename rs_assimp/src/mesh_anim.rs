use crate::mesh_key::MeshKey;
use std::marker::PhantomData;

pub struct MeshAnim<'a> {
    c: &'a mut russimp_sys::aiMeshAnim,
    pub name: String,
    pub mesh_keys: Vec<MeshKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshAnim<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMeshAnim) -> MeshAnim<'a> {
        let name = c.mName.into();
        let mesh_keys = unsafe { std::slice::from_raw_parts_mut(c.mKeys, c.mNumKeys as _) };
        let mesh_keys = mesh_keys
            .iter_mut()
            .map(|x| MeshKey::borrow_from(x))
            .collect();
        MeshAnim {
            c,
            name,
            mesh_keys,
            marker: PhantomData,
        }
    }
}
