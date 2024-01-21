pub mod bake_info;
pub mod base_compute_pipeline;
pub mod base_render_pipeline;
pub mod bind_group_layout_entry_hook;
pub mod command;
pub mod default_textures;
pub mod depth_texture;
pub mod egui_render;
pub mod error;
pub mod gpu_buffer;
pub mod gpu_vertex_buffer;
pub mod reflection;
pub mod render_pipeline;
pub mod renderer;
pub mod shader_library;
pub mod vertex_data_type;
pub mod wgpu_context;

#[derive(Debug)]
pub enum VertexBufferType {
    Interleaved(type_layout::TypeLayoutInfo),
    Noninterleaved,
}
