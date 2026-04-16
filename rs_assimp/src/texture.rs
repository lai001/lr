use crate::convert::ConvertToString;
use rs_assimp_sys::*;
use std::marker::PhantomData;

pub struct Texture<'a> {
    _ai_texture: &'a mut aiTexture,
    pub width: u32,
    pub height: u32,
    pub ach_format_hint: [i8; 9usize],
    pub pc_data: &'a mut aiTexel,
    pub filename: String,
    marker: PhantomData<&'a ()>,
}

impl<'a> Texture<'a> {
    pub fn borrow_from(ai_texture: &'a mut aiTexture) -> Texture<'a> {
        let width = ai_texture.mWidth;
        let height = ai_texture.mHeight;
        let ach_format_hint = ai_texture.achFormatHint;
        let pc_data = ai_texture.pcData;
        let filename = ai_texture.mFilename.to_string();

        Texture {
            _ai_texture: ai_texture,
            width,
            height,
            ach_format_hint,
            pc_data: unsafe { pc_data.as_mut().unwrap() },
            filename,
            marker: PhantomData,
        }
    }
}
