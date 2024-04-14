#[derive(Clone, Hash, PartialEq, Eq)]
pub struct UniformHook {
    pub has_dynamic_offset: bool,
    pub min_binding_size: Option<wgpu::BufferSize>,
    pub count: Option<std::num::NonZeroU32>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TextureSampleTypeHook {
    pub sample_type: wgpu::TextureSampleType,
    pub count: Option<std::num::NonZeroU32>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum EBindGroupLayoutEntryHookType {
    Uniform(UniformHook),
    TextureSampleType(TextureSampleTypeHook),
    SamplerBindingType(wgpu::SamplerBindingType),
}
