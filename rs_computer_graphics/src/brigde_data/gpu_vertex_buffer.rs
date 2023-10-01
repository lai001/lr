pub trait TGpuVertexBuffer: Sized {
    fn get_vertex_buffers(&self) -> &[wgpu::Buffer];

    fn get_vertex_count(&self) -> u32;

    fn get_index_buffer(&self) -> Option<&wgpu::Buffer>;

    fn get_index_count(&self) -> Option<u32>;
}
