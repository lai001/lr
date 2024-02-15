pub mod acceleration_bake;
pub mod bake_info;
pub mod base_compute_pipeline;
pub mod base_render_pipeline;
pub mod bind_group_layout_entry_hook;
pub mod buffer_dimensions;
pub mod command;
pub mod compute_pipeline;
pub mod cube_map;
pub mod default_textures;
pub mod depth_texture;
pub mod egui_render;
pub mod error;
pub mod global_shaders;
pub mod gpu_buffer;
pub mod gpu_vertex_buffer;
pub mod ibl_readback;
pub mod reflection;
pub mod render_pipeline;
#[cfg(feature = "renderdoc")]
pub mod renderdoc;
pub mod renderer;
pub mod sampler_cache;
pub mod shader_library;
pub mod texture_loader;
pub mod texture_readback;
pub mod vertex_data_type;
pub mod wgpu_context;

#[derive(Debug)]
pub enum VertexBufferType {
    Interleaved(type_layout::TypeLayoutInfo),
    Noninterleaved,
}

pub(crate) fn get_cargo_manifest_dir() -> std::path::PathBuf {
    const CARGO_MANIFEST_DIR: &'static str = env!("CARGO_MANIFEST_DIR");
    CARGO_MANIFEST_DIR.into()
}

pub(crate) fn get_buildin_shader_dir() -> std::path::PathBuf {
    get_cargo_manifest_dir().join("shaders")
}
