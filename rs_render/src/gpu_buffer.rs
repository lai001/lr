pub mod uniform {
    pub fn from<T>(device: &wgpu::Device, data: &T, label: Option<&str>) -> wgpu::Buffer {
        let unsafe_uniform_raw_buffer: &[u8] = unsafe {
            std::slice::from_raw_parts((data as *const T) as *const u8, std::mem::size_of::<T>())
        };
        let uniform_buf = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: unsafe_uniform_raw_buffer,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::MAP_READ,
            },
        );
        uniform_buf
    }

    pub fn from_array<T>(device: &wgpu::Device, data: &[T], label: Option<&str>) -> wgpu::Buffer {
        let unsafe_uniform_raw_buffer: &[u8] = rs_foundation::cast_to_raw_buffer(data);
        let uniform_buf = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: unsafe_uniform_raw_buffer,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::MAP_READ,
            },
        );
        uniform_buf
    }
}

pub mod vertex {
    pub fn from<T>(device: &wgpu::Device, vertex: &[T], label: Option<&str>) -> wgpu::Buffer {
        let vertex_buf = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: rs_foundation::cast_to_raw_buffer(vertex),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        vertex_buf
    }
}

pub mod index {
    pub fn from(device: &wgpu::Device, index_data: &[u32], label: Option<&str>) -> wgpu::Buffer {
        let index_data_raw_buffer = rs_foundation::cast_to_raw_buffer(index_data);
        let index_buf = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: index_data_raw_buffer,
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        index_buf
    }
}
