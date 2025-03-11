// pub trait TGpuVertexBuffer: Sized {
//     fn get_vertex_buffers(&self) -> &[wgpu::Buffer];

//     fn get_vertex_count(&self) -> u32;

//     fn get_index_buffer(&self) -> Option<&wgpu::Buffer>;

//     fn get_index_count(&self) -> Option<u32>;
// }

use std::sync::Arc;

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
pub enum EMultipleThreadingDrawCallType {
    MultiDrawIndirect(MutilpleThreadingMultiDrawIndirect),
    Draw(Draw),
}

impl EMultipleThreadingDrawCallType {
    pub fn to_local<'a>(&'a self) -> EDrawCallType<'a> {
        match self {
            EMultipleThreadingDrawCallType::MultiDrawIndirect(x) => {
                EDrawCallType::MultiDrawIndirect(MultiDrawIndirect {
                    indirect_buffer: &x.indirect_buffer,
                    indirect_offset: x.indirect_offset,
                    count: x.count,
                })
            }
            EMultipleThreadingDrawCallType::Draw(x) => EDrawCallType::Draw(Draw {
                instances: x.instances.clone(),
            }),
        }
    }
}

#[derive(Clone)]
pub struct GpuVertexBufferImp<'a> {
    pub vertex_buffers: &'a [&'a wgpu::Buffer],
    pub vertex_count: u32,
    pub index_buffer: Option<&'a wgpu::Buffer>,
    pub index_count: Option<u32>,
    pub draw_type: EDrawCallType<'a>,
}

#[derive(Clone)]
pub struct MutilpleThreadingMultiDrawIndirect {
    pub indirect_buffer: Arc<wgpu::Buffer>,
    pub indirect_offset: wgpu::BufferAddress,
    pub count: u32,
}

#[derive(Clone)]
pub struct MultipleThreadingGpuVertexBufferImp {
    pub vertex_buffers: Vec<Arc<wgpu::Buffer>>,
    pub vertex_count: u32,
    pub index_buffer: Option<Arc<wgpu::Buffer>>,
    pub index_count: Option<u32>,
    pub draw_type: EMultipleThreadingDrawCallType,
}
