use std::marker::PhantomData;

pub struct MeshKey<'a> {
    c: &'a mut russimp_sys::aiMeshKey,
    pub time: f64,
    pub value: u32,
    marker: PhantomData<&'a ()>,
}

impl<'a> MeshKey<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMeshKey) -> MeshKey<'a> {
        let time = c.mTime;
        let value = c.mValue;
        MeshKey {
            c,
            time,
            value,
            marker: PhantomData,
        }
    }
}
