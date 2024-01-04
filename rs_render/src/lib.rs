pub mod base_compute_pipeline;
pub mod base_render_pipeline;
pub mod bind_group_layout_entry_hook;
pub mod egui_render;
pub mod error;
pub mod gpu_vertex_buffer;
pub mod reflection;
pub mod renderer;
pub mod shader_library;
pub mod wgpu_context;

#[derive(Debug)]
pub enum VertexBufferType {
    Interleaved(type_layout::TypeLayoutInfo),
    Noninterleaved,
}
