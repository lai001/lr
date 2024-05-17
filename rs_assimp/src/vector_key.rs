use std::marker::PhantomData;

pub struct VectorKey<'a> {
    _ai_vector_key: &'a mut russimp_sys::aiVectorKey,
    pub time: f64,
    pub value: glam::Vec3,
    marker: PhantomData<&'a ()>,
}

impl<'a> VectorKey<'a> {
    pub fn borrow_from(ai_vector_key: &'a mut russimp_sys::aiVectorKey) -> VectorKey<'a> {
        let time = ai_vector_key.mTime;
        let value = ai_vector_key.mValue;
        let value = glam::vec3(value.x, value.y, value.z);
        VectorKey {
            _ai_vector_key: ai_vector_key,
            time,
            value,
            marker: PhantomData,
        }
    }
}
