pub trait TGpuVertexBuffer: Sized {
    fn get_vertex_buffer(&self, slot: u32) -> &wgpu::Buffer;

    fn get_index_buffer(&self) -> &wgpu::Buffer;

    fn get_index_count(&self) -> u32;
}
