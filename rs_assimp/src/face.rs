use std::marker::PhantomData;

pub struct Face<'a> {
    _ai_face: &'a mut russimp_sys::aiFace,
    pub indices: Vec<u32>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Face<'a> {
    pub fn borrow_from(ai_face: &'a mut russimp_sys::aiFace) -> Face<'a> {
        let ai_indices =
            unsafe { std::slice::from_raw_parts(ai_face.mIndices, ai_face.mNumIndices as _) };
        let indices = ai_indices.to_vec();
        Face {
            _ai_face: ai_face,
            indices,
            marker: PhantomData,
        }
    }
}
