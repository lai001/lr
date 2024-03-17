use std::marker::PhantomData;

pub struct MeshMorphKey<'a> {
    c: &'a mut russimp_sys::aiMeshMorphKey,
    pub time: f64,
    pub values: Vec<u32>,
    pub weights: Vec<f64>,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshMorphKey<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMeshMorphKey) -> MeshMorphKey<'a> {
        let time = c.mTime;
        let values =
            unsafe { std::slice::from_raw_parts_mut(c.mValues, c.mNumValuesAndWeights as _) }
                .to_vec();
        let weights =
            unsafe { std::slice::from_raw_parts_mut(c.mWeights, c.mNumValuesAndWeights as _) }
                .to_vec();
        MeshMorphKey {
            c,
            time,
            marker: PhantomData,
            values,
            weights,
        }
    }
}
