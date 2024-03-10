use std::marker::PhantomData;

pub struct Face<'a> {
    c: &'a mut russimp_sys::aiFace,
    pub indices: Vec<u32>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Face<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiFace) -> Face<'a> {
        let ai_indices = unsafe { std::slice::from_raw_parts(c.mIndices, c.mNumIndices as _) };
        let indices = ai_indices.to_vec();
        Face {
            c,
            indices,
            marker: PhantomData,
        }
    }
}
