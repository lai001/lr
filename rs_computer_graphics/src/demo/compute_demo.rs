use crate::shader::shader_library::ShaderLibrary;

pub struct ComputeDemo {
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputeDemo {
    pub fn new(device: &wgpu::Device) -> ComputeDemo {
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("collatz_compute.wgsl");

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: "main",
        });
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);

        ComputeDemo {
            compute_pipeline,
            bind_group_layout,
        }
    }

    pub fn execute(
        &self,
        numbers: &Vec<i32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<u32> {
        let reuslt = pollster::block_on(self.execute_inner(numbers, device, queue));
        reuslt.unwrap()
    }

    async fn execute_inner(
        &self,
        numbers: &Vec<i32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Vec<u32>> {
        let buffer = crate::util::cast_to_raw_buffer(numbers);

        let storage_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: buffer,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            },
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(numbers.len() as u32, 1, 1);
        }

        let size = (numbers.len() * std::mem::size_of::<u32>()) as wgpu::BufferAddress;
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);

        queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);

        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(_)) = receiver.recv() {
            let data = buffer_slice.get_mapped_range();
            let result: Vec<u32> =
                crate::util::cast_to_raw_type_buffer(data.as_ptr(), data.len()).to_vec();
            drop(data);
            staging_buffer.unmap();
            Some(result)
        } else {
            panic!()
        }
    }
}
