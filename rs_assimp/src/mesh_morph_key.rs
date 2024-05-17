use std::marker::PhantomData;

pub struct MeshMorphKey<'a> {
    _ai_mesh_morph_key: &'a mut russimp_sys::aiMeshMorphKey,
    pub time: f64,
    pub values: Vec<u32>,
    pub weights: Vec<f64>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshMorphKey<'a> {
    pub fn borrow_from(ai_mesh_morph_key: &'a mut russimp_sys::aiMeshMorphKey) -> MeshMorphKey<'a> {
        let time = ai_mesh_morph_key.mTime;
        let values = unsafe {
            std::slice::from_raw_parts_mut(
                ai_mesh_morph_key.mValues,
                ai_mesh_morph_key.mNumValuesAndWeights as _,
            )
        }
        .to_vec();
        let weights = unsafe {
            std::slice::from_raw_parts_mut(
                ai_mesh_morph_key.mWeights,
                ai_mesh_morph_key.mNumValuesAndWeights as _,
            )
        }
        .to_vec();
        MeshMorphKey {
            _ai_mesh_morph_key: ai_mesh_morph_key,
            time,
            marker: PhantomData,
            values,
            weights,
        }
    }
}
