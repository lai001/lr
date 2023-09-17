use super::gpu_vertex_buffer::TGpuVertexBuffer;
use crate::util::{create_gpu_index_buffer_from, create_gpu_vertex_buffer_from};
use type_layout::TypeLayout;

// #[repr(C, packed)]
#[repr(C)]
#[derive(Clone, Copy, Debug, TypeLayout)]
pub struct ColorVertex {
    pub vertex_color: glam::Vec4,
    pub position: glam::Vec3,
}

impl ColorVertex {
    pub fn new(vertex_color: glam::Vec4, position: glam::Vec3) -> ColorVertex {
        ColorVertex {
            vertex_color,
            position,
        }
    }
}

pub struct ColorVertexCollection {
    vertex_buffer: Vec<ColorVertex>,
    index_buffer: Vec<u32>,
}

impl ColorVertexCollection {
    pub fn new(vertex_buffer: Vec<ColorVertex>, index_buffer: Vec<u32>) -> ColorVertexCollection {
        ColorVertexCollection {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn get_vertex_buffer(&self) -> &[ColorVertex] {
        self.vertex_buffer.as_ref()
    }

    pub fn get_index_buffer(&self) -> &[u32] {
        self.index_buffer.as_ref()
    }
}

pub struct ColorVertexBuffer {
    vertex_buffer: Vec<wgpu::Buffer>,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl TGpuVertexBuffer for ColorVertexBuffer {
    fn get_vertex_buffer(&self, slot: u32) -> &wgpu::Buffer {
        self.vertex_buffer
            .get(slot as usize)
            .expect(&format!("invalid slot: {}", slot))
    }

    fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    fn get_index_count(&self) -> u32 {
        self.index_count
    }
}

impl ColorVertexBuffer {
    pub fn from_interleaved(
        device: &wgpu::Device,
        line3d_collection: &ColorVertexCollection,
    ) -> ColorVertexBuffer {
        let vertex_buffer = create_gpu_vertex_buffer_from(
            device,
            &line3d_collection.vertex_buffer,
            Some("ColorVertexBuffer.vertex_buffer"),
        );
        let index_buffer = create_gpu_index_buffer_from(
            device,
            &line3d_collection.index_buffer,
            Some("ColorVertexBuffer.index_buffer"),
        );
        let buffer = ColorVertexBuffer {
            vertex_buffer: vec![vertex_buffer],
            index_buffer,
            index_count: line3d_collection.index_buffer.len() as u32,
        };
        buffer
    }

    pub fn from_noninterleaved(
        device: &wgpu::Device,
        vertex_colors: Vec<glam::Vec4>,
        positions: Vec<glam::Vec3>,
        index_buffer: Vec<u32>,
    ) -> ColorVertexBuffer {
        let index_count = index_buffer.len() as u32;
        let vertex_color_buffer = create_gpu_vertex_buffer_from(
            device,
            &vertex_colors,
            Some("ColorVertexBuffer.vertex_color_buffer"),
        );
        let position_buffer = create_gpu_vertex_buffer_from(
            device,
            &positions,
            Some("ColorVertexBuffer.position_buffer"),
        );
        let index_buffer = create_gpu_index_buffer_from(
            device,
            &index_buffer,
            Some("ColorVertexBuffer.index_buffer"),
        );
        let buffer = ColorVertexBuffer {
            vertex_buffer: vec![vertex_color_buffer, position_buffer],
            index_buffer,
            index_count,
        };
        buffer
    }
}
