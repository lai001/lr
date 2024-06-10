use crate::command::{SamplerHandle, TextureHandle};

#[derive(Debug, Clone)]
pub struct MSAAInfo {
    pub texture: TextureHandle,
    pub depth_texture: TextureHandle,
}

#[derive(Debug, Clone)]
pub struct FXAAInfo {
    pub sampler: SamplerHandle,
    pub texture: TextureHandle,
}

#[derive(Debug, Clone)]
pub enum EAntialiasType {
    None,
    FXAA(FXAAInfo),
    MSAA(MSAAInfo),
}
