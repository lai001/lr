use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
};
use wgpu::*;

pub struct SamplerCache {
    cache: HashMap<u64, Arc<Sampler>>,
}

impl SamplerCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn create_sampler(
        &mut self,
        device: &Device,
        desc: &SamplerDescriptor<'_>,
    ) -> Arc<Sampler> {
        let mut state = DefaultHasher::new();
        desc.label.hash(&mut state);
        desc.address_mode_u.hash(&mut state);
        desc.address_mode_v.hash(&mut state);
        desc.address_mode_w.hash(&mut state);
        desc.mag_filter.hash(&mut state);
        desc.min_filter.hash(&mut state);
        desc.mipmap_filter.hash(&mut state);
        desc.lod_min_clamp.to_bits().hash(&mut state);
        desc.lod_max_clamp.to_bits().hash(&mut state);
        desc.compare.hash(&mut state);
        desc.anisotropy_clamp.hash(&mut state);
        desc.border_color.hash(&mut state);
        let key = state.finish();
        if !self.cache.contains_key(&key) {
            let sampler = device.create_sampler(desc);
            self.cache.insert(key, Arc::new(sampler));
        }
        return self.cache.get(&key).unwrap().clone();
    }
}
