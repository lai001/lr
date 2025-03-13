use crate::{base_render_pipeline::BaseRenderPipeline, command::EBindingResource};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(crate) fn find_or_insert_bind_groups(
    device: &wgpu::Device,
    pipeline: &BaseRenderPipeline,
    texture_views: &HashMap<u64, &wgpu::TextureView>,
    buffers: &HashMap<u64, Arc<wgpu::Buffer>>,
    samplers: &HashMap<u64, wgpu::Sampler>,
    binding_resources_group: &Vec<Vec<EBindingResource>>,
    bind_groups_collection: &mut moka::sync::Cache<u64, Arc<Vec<wgpu::BindGroup>>>,
) -> crate::error::Result<Arc<Vec<wgpu::BindGroup>>> {
    let mut hasher = std::hash::DefaultHasher::new();
    let mut group_binding_resource: Vec<Vec<wgpu::BindingResource>> =
        Vec::with_capacity(binding_resources_group.len());

    for (group, binding_resource) in binding_resources_group.iter().enumerate() {
        let mut binding_resources: Vec<wgpu::BindingResource> =
            Vec::with_capacity(binding_resource.len());
        for (binding, binding_resource_type) in binding_resource.iter().enumerate() {
            match binding_resource_type {
                EBindingResource::Texture(handle) => {
                    handle.hash(&mut hasher);
                    let texture_view =
                        texture_views
                            .get(handle)
                            .ok_or(crate::error::Error::Other(Some(format!(
                                "{}, {}, texture view is null",
                                group, binding
                            ))))?;
                    binding_resources.push(wgpu::BindingResource::TextureView(texture_view));
                }
                EBindingResource::Constants(buffer_handle) => {
                    buffer_handle.hash(&mut hasher);
                    let buffer = buffers
                        .get(buffer_handle)
                        .ok_or(crate::error::Error::Other(Some(format!(
                            "{}, {}, constants is null",
                            group, binding
                        ))))
                        .expect("Texture should not be null")
                        .as_entire_binding();
                    binding_resources.push(buffer);
                }
                EBindingResource::Sampler(handle) => {
                    handle.hash(&mut hasher);
                    let sampler =
                        samplers
                            .get(handle)
                            .ok_or(crate::error::Error::Other(Some(format!(
                                "{}, {}, sampler is null",
                                group, binding
                            ))))?;
                    binding_resources.push(wgpu::BindingResource::Sampler(sampler));
                }
            }
        }
        group_binding_resource.push(binding_resources);
    }

    let cache_key = hasher.finish();
    if !bind_groups_collection.contains_key(&cache_key) {
        bind_groups_collection.insert(
            cache_key,
            Arc::new(pipeline.make_bind_groups_binding_resources(device, group_binding_resource)),
        );
    }

    Ok(bind_groups_collection.get(&cache_key).unwrap())
}

pub fn find_most_compatible_texture_usages(format: wgpu::TextureFormat) -> wgpu::TextureUsages {
    let _ = format;
    return wgpu::TextureUsages::all() - wgpu::TextureUsages::STORAGE_ATOMIC;
}

pub fn is_compatible(format: wgpu::TextureFormat, usages: wgpu::TextureUsages) -> bool {
    if format == wgpu::TextureFormat::Rgba8Unorm {
        if usages.contains(wgpu::TextureUsages::STORAGE_ATOMIC) {
            return false;
        }
    }
    return true;
}
