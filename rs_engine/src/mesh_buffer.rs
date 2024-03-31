use crate::handle::BufferHandle;

pub struct MeshBuffer {
    pub vertex_buffers: Vec<BufferHandle>,
    pub vertex_count: u32,
    pub index_buffer: Option<BufferHandle>,
    pub index_count: Option<u32>,
}
