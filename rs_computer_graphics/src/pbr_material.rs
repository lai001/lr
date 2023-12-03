use std::sync::Arc;
use wgpu::*;

pub struct PBRMaterial {
    albedo_texture: Arc<Option<wgpu::Texture>>,
    normal_texture: Arc<Option<wgpu::Texture>>,
    metallic_texture: Arc<Option<wgpu::Texture>>,
    roughness_texture: Arc<Option<wgpu::Texture>>,
    brdflut_texture: Arc<Option<wgpu::Texture>>,
    pre_filter_cube_map_texture: Arc<Option<wgpu::Texture>>,
    irradiance_texture: Arc<Option<wgpu::Texture>>,
}

impl PBRMaterial {
    pub fn new(
        albedo_texture: Arc<Option<wgpu::Texture>>,
        normal_texture: Arc<Option<wgpu::Texture>>,
        metallic_texture: Arc<Option<wgpu::Texture>>,
        roughness_texture: Arc<Option<wgpu::Texture>>,
        brdflut_texture: Arc<Option<wgpu::Texture>>,
        pre_filter_cube_map_texture: Arc<Option<wgpu::Texture>>,
        irradiance_texture: Arc<Option<wgpu::Texture>>,
    ) -> PBRMaterial {
        PBRMaterial {
            albedo_texture,
            normal_texture,
            metallic_texture,
            roughness_texture,
            brdflut_texture,
            pre_filter_cube_map_texture,
            irradiance_texture,
        }
    }

    pub fn get_albedo_texture_view(&self) -> wgpu::TextureView {
        match *self.albedo_texture {
            Some(ref albedo_texture) => {
                return albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());
            }
            None => {
                panic!()
            }
        }
    }

    pub fn get_normal_texture_view(&self) -> wgpu::TextureView {
        match Option::as_ref(&self.normal_texture) {
            Some(normal_texture) => {
                return normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
            }
            None => {
                panic!()
            }
        }
    }

    pub fn get_metallic_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref metallic_texture) = *self.metallic_texture {
            return metallic_texture.create_view(&wgpu::TextureViewDescriptor::default());
        } else {
            panic!()
        }
    }

    pub fn get_roughness_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref roughness_texture) = *self.roughness_texture {
            return roughness_texture.create_view(&wgpu::TextureViewDescriptor::default());
        } else {
            panic!()
        }
    }

    pub fn get_brdflut_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref brdflut_texture) = *self.brdflut_texture {
            return brdflut_texture.create_view(&wgpu::TextureViewDescriptor::default());
        } else {
            panic!()
        }
    }

    pub fn get_pre_filter_cube_map_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref pre_filter_cube_map_texture) = *self.pre_filter_cube_map_texture {
            let mip_level_count = pre_filter_cube_map_texture.mip_level_count();
            let array_layer_count = pre_filter_cube_map_texture.depth_or_array_layers();
            return pre_filter_cube_map_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(pre_filter_cube_map_texture.format()),
                dimension: Some(TextureViewDimension::Cube),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(mip_level_count),
                base_array_layer: 0,
                array_layer_count: Some(array_layer_count),
            });
        } else {
            panic!()
        }
    }

    pub fn get_irradiance_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref irradiance_texture) = *self.irradiance_texture {
            let mip_level_count = irradiance_texture.mip_level_count();
            let array_layer_count = irradiance_texture.depth_or_array_layers();
            return irradiance_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(irradiance_texture.format()),
                dimension: Some(TextureViewDimension::Cube),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(mip_level_count),
                base_array_layer: 0,
                array_layer_count: Some(array_layer_count),
            });
        } else {
            panic!()
        }
    }
}
