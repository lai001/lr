use std::marker::PhantomData;

pub struct QuatKey<'a> {
    c: &'a mut russimp_sys::aiQuatKey,
    pub time: f64,
    pub value: glam::Quat,
    marker: PhantomData<&'a ()>,
}

impl<'a> QuatKey<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiQuatKey) -> QuatKey<'a> {
        let time = c.mTime;
        let value = c.mValue;
        let value = glam::quat(value.x, value.y, value.z, value.w);
        QuatKey {
            c,
            time,
            value,
            marker: PhantomData,
        }
    }
}
