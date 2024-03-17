use std::marker::PhantomData;

pub struct VectorKey<'a> {
    c: &'a mut russimp_sys::aiVectorKey,
    pub time: f64,
    pub value: glam::Vec3,
    marker: PhantomData<&'a ()>,
}

impl<'a> VectorKey<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiVectorKey) -> VectorKey<'a> {
        let time = c.mTime;
        let value = c.mValue;
        let value = glam::vec3(value.x, value.y, value.z);
        VectorKey {
            c,
            time,
            value,
            marker: PhantomData,
        }
    }
}
