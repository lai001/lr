use crate::node_anim::EAnimInterpolation;
use rs_assimp_sys::*;
use std::marker::PhantomData;

pub struct QuatKey<'a> {
    _ai_quat_key: &'a mut aiQuatKey,
    pub time: f64,
    pub value: glam::Quat,
    pub interpolation: EAnimInterpolation,
    marker: PhantomData<&'a ()>,
}

impl<'a> QuatKey<'a> {
    pub fn borrow_from(ai_quat_key: &'a mut aiQuatKey) -> QuatKey<'a> {
        let time = ai_quat_key.mTime;
        let value = ai_quat_key.mValue;
        let value = glam::quat(value.x, value.y, value.z, value.w);
        let interpolation = ai_quat_key.mInterpolation.try_into().unwrap();
        QuatKey {
            _ai_quat_key: ai_quat_key,
            time,
            value,
            marker: PhantomData,
            interpolation,
        }
    }
}
