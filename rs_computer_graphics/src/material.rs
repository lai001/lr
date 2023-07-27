use std::sync::Arc;

use crate::default_textures::DefaultTextures;

pub struct Material {
    diffuse_texture: Arc<Option<wgpu::Texture>>,
    specular_texture: Arc<Option<wgpu::Texture>>,
}

impl Material {
    pub fn new(
        diffuse_texture: Arc<Option<wgpu::Texture>>,
        specular_texture: Arc<Option<wgpu::Texture>>,
    ) -> Material {
        Material {
            diffuse_texture,
            specular_texture,
        }
    }

    pub fn get_diffuse_texture(&self) -> Arc<Option<wgpu::Texture>> {
        self.diffuse_texture.clone()
    }
    pub fn get_specular_texture(&self) -> Arc<Option<wgpu::Texture>> {
        self.specular_texture.clone()
    }

    pub fn get_diffuse_texture_view(&self) -> wgpu::TextureView {
        match self.diffuse_texture.clone().as_ref() {
            Some(texture) => texture.create_view(&wgpu::TextureViewDescriptor::default()),
            None => DefaultTextures::default()
                .lock()
                .unwrap()
                .get_black_texture_view(),
        }
    }

    pub fn get_specular_texture_view(&self) -> wgpu::TextureView {
        match self.specular_texture.clone().as_ref() {
            Some(texture) => texture.create_view(&wgpu::TextureViewDescriptor::default()),
            None => DefaultTextures::default()
                .lock()
                .unwrap()
                .get_black_texture_view(),
        }
    }

    pub fn set_diffuse_texture(&mut self, diffuse_texture: Arc<Option<wgpu::Texture>>) {
        self.diffuse_texture = diffuse_texture;
    }

    pub fn set_specular_texture(&mut self, specular_texture: Arc<Option<wgpu::Texture>>) {
        self.specular_texture = specular_texture;
    }
}
