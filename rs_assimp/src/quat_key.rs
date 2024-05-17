use std::marker::PhantomData;

pub struct QuatKey<'a> {
    _ai_quat_key: &'a mut russimp_sys::aiQuatKey,
    pub time: f64,
    pub value: glam::Quat,
    marker: PhantomData<&'a ()>,
}

impl<'a> QuatKey<'a> {
    pub fn borrow_from(ai_quat_key: &'a mut russimp_sys::aiQuatKey) -> QuatKey<'a> {
        let time = ai_quat_key.mTime;
        let value = ai_quat_key.mValue;
        let value = glam::quat(value.x, value.y, value.z, value.w);
        QuatKey {
            _ai_quat_key: ai_quat_key,
            time,
            value,
            marker: PhantomData,
        }
    }
}
