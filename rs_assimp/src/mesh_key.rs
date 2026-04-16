use rs_assimp_sys::*;
use std::marker::PhantomData;

pub struct MeshKey<'a> {
    _ai_mesh_key: &'a mut aiMeshKey,
    pub time: f64,
    pub value: u32,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshKey<'a> {
    pub fn borrow_from(ai_mesh_key: &'a mut aiMeshKey) -> MeshKey<'a> {
        let time = ai_mesh_key.mTime;
        let value = ai_mesh_key.mValue;
        MeshKey {
            _ai_mesh_key: ai_mesh_key,
            time,
            value,
            marker: PhantomData,
        }
    }
}
