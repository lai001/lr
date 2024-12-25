use std::{collections::HashMap, sync::Arc};
use wgpu::util::DeviceExt;

pub(crate) struct GetResult {
    pub(crate) cluster_lights_buffer: Arc<wgpu::Buffer>,
    pub(crate) cluster_light_indices_buffer: Arc<wgpu::Buffer>,
}

pub(crate) struct LightCulling {
    pub(crate) cluster_lights_pool: HashMap<usize, Arc<wgpu::Buffer>>,
    pub(crate) cluster_light_indices_pool: HashMap<usize, Arc<wgpu::Buffer>>,
}

impl LightCulling {
    pub(crate) fn new() -> LightCulling {
        LightCulling {
            cluster_lights_pool: HashMap::new(),
            cluster_light_indices_pool: HashMap::new(),
        }
    }

    pub(crate) fn get_or_add(
        &mut self,
        device: &wgpu::Device,
        size: usize,
        point_lights_num: usize,
    ) -> GetResult {
        let cluster_lights_buffer = self
            .cluster_lights_pool
            .entry(size)
            .or_insert_with(|| {
                let cluster_lights: Vec<u32> = vec![0; size * point_lights_num];
                let cluster_lights_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: rs_foundation::cast_to_raw_buffer(&cluster_lights),
                        usage: wgpu::BufferUsages::STORAGE,
                    });
                Arc::new(cluster_lights_buffer)
            })
            .clone();

        let cluster_light_indices_buffer = self
            .cluster_light_indices_pool
            .entry(size)
            .or_insert_with(|| {
                let cluster_light_indices =
                    vec![crate::constants::ClusterLightIndex::default(); size];
                let cluster_light_indices_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: rs_foundation::cast_to_raw_buffer(&cluster_light_indices),
                        usage: wgpu::BufferUsages::STORAGE,
                    });
                Arc::new(cluster_light_indices_buffer)
            })
            .clone();
        GetResult {
            cluster_lights_buffer,
            cluster_light_indices_buffer,
        }
    }
}
