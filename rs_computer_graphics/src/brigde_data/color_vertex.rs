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
    index_buffer: Option<wgpu::Buffer>,
    index_count: Option<u32>,
    vertex_count: u32,
}

impl TGpuVertexBuffer for ColorVertexBuffer {
    fn get_vertex_buffers(&self) -> &[wgpu::Buffer] {
        &self.vertex_buffer
    }

    fn get_index_buffer(&self) -> Option<&wgpu::Buffer> {
        if let Some(index_buffer) = &self.index_buffer {
            Some(index_buffer)
        } else {
            None
        }
    }

    fn get_index_count(&self) -> Option<u32> {
        if let Some(index_count) = self.index_count {
            Some(index_count)
        } else {
            None
        }
    }

    fn get_vertex_count(&self) -> u32 {
        self.vertex_count
    }
}

impl ColorVertexBuffer {
    pub fn from_interleaved(
        device: &wgpu::Device,
        vertex_buffer: &[ColorVertex],
    ) -> ColorVertexBuffer {
        let vertex_count = vertex_buffer.len() as u32;
        let vertex_buffer = create_gpu_vertex_buffer_from(
            device,
            &vertex_buffer,
            Some("ColorVertexBuffer.vertex_buffer"),
        );
        let buffer = ColorVertexBuffer {
            vertex_buffer: vec![vertex_buffer],
            index_buffer: None,
            index_count: None,
            vertex_count,
        };
        buffer
    }

    pub fn from_interleaved_indexed(
        device: &wgpu::Device,
        vertex_buffer: &[ColorVertex],
        index_buffer: &[u32],
    ) -> ColorVertexBuffer {
        let vertex_count = vertex_buffer.len() as u32;
        let index_count = index_buffer.len() as u32;
        let vertex_buffer = create_gpu_vertex_buffer_from(
            device,
            &vertex_buffer,
            Some("ColorVertexBuffer.vertex_buffer"),
        );
        let index_buffer = create_gpu_index_buffer_from(
            device,
            &index_buffer,
            Some("ColorVertexBuffer.index_buffer"),
        );
        let buffer = ColorVertexBuffer {
            vertex_buffer: vec![vertex_buffer],
            index_buffer: Some(index_buffer),
            index_count: Some(index_count),
            vertex_count,
        };
        buffer
    }

    pub fn from_noninterleaved_indexed(
        device: &wgpu::Device,
        vertex_colors: Vec<glam::Vec4>,
        positions: Vec<glam::Vec3>,
        index_buffer: Vec<u32>,
    ) -> ColorVertexBuffer {
        debug_assert_eq!(vertex_colors.len(), positions.len());
        let index_count = index_buffer.len() as u32;
        let vertex_count = vertex_colors.len() as u32;
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
            index_buffer: Some(index_buffer),
            index_count: Some(index_count),
            vertex_count,
        };
        buffer
    }

    pub fn from_noninterleaved(
        device: &wgpu::Device,
        vertex_colors: &[glam::Vec4],
        positions: &[glam::Vec3],
    ) -> ColorVertexBuffer {
        debug_assert_eq!(vertex_colors.len(), positions.len());
        let vertex_count = vertex_colors.len() as u32;
        let vertex_color_buffer = create_gpu_vertex_buffer_from(
            device,
            vertex_colors,
            Some("ColorVertexBuffer.vertex_color_buffer"),
        );
        let position_buffer = create_gpu_vertex_buffer_from(
            device,
            positions,
            Some("ColorVertexBuffer.position_buffer"),
        );
        let buffer = ColorVertexBuffer {
            vertex_buffer: vec![vertex_color_buffer, position_buffer],
            index_buffer: None,
            index_count: None,
            vertex_count,
        };
        buffer
    }
}
