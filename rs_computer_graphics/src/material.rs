use std::sync::Arc;

use crate::default_textures::DefaultTextures;

pub struct Material {
    diffuse_texture: Option<Arc<wgpu::Texture>>,
    specular_texture: Option<Arc<wgpu::Texture>>,
}

impl Material {
    pub fn new(
        diffuse_texture: Option<Arc<wgpu::Texture>>,
        specular_texture: Option<Arc<wgpu::Texture>>,
    ) -> Material {
        Material {
            diffuse_texture,
            specular_texture,
        }
    }

    pub fn get_diffuse_texture(&self) -> Option<Arc<wgpu::Texture>> {
        match &self.diffuse_texture {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
    pub fn get_specular_texture(&self) -> Option<Arc<wgpu::Texture>> {
        match &self.specular_texture {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub fn get_diffuse_texture_view(&self) -> wgpu::TextureView {
        let texture = self.get_diffuse_texture().unwrap_or(
            DefaultTextures::default()
                .lock()
                .unwrap()
                .get_black_texture(),
        );
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn get_specular_texture_view(&self) -> wgpu::TextureView {
        let texture = self.get_specular_texture().unwrap_or(
            DefaultTextures::default()
                .lock()
                .unwrap()
                .get_black_texture(),
        );
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn set_diffuse_texture(&mut self, diffuse_texture: Option<Arc<wgpu::Texture>>) {
        self.diffuse_texture = diffuse_texture;
    }

    pub fn set_specular_texture(&mut self, specular_texture: Option<Arc<wgpu::Texture>>) {
        self.specular_texture = specular_texture;
    }
}
