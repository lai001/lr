// pub trait TGpuVertexBuffer: Sized {
//     fn get_vertex_buffers(&self) -> &[wgpu::Buffer];

//     fn get_vertex_count(&self) -> u32;

//     fn get_index_buffer(&self) -> Option<&wgpu::Buffer>;

//     fn get_index_count(&self) -> Option<u32>;
// }

#[derive(Clone)]
pub struct MultiDrawIndirect<'a> {
    pub indirect_buffer: &'a wgpu::Buffer,
    pub indirect_offset: wgpu::BufferAddress,
    pub count: u32,
}

#[derive(Clone)]
pub struct Draw {
    pub instances: std::ops::Range<u32>,
}

#[derive(Clone)]
pub enum EDrawCallType<'a> {
    MultiDrawIndirect(MultiDrawIndirect<'a>),
    Draw(Draw),
}

#[derive(Clone)]
pub struct GpuVertexBufferImp<'a> {
    pub vertex_buffers: &'a [&'a wgpu::Buffer],
    pub vertex_count: u32,
    pub index_buffer: Option<&'a wgpu::Buffer>,
    pub index_count: Option<u32>,
    pub draw_type: EDrawCallType<'a>,
}
