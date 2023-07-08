use std::sync::{Arc, Mutex};

use crate::util;

pub struct DefaultTextures {
    black: Option<Arc<wgpu::Texture>>,
    white: Option<Arc<wgpu::Texture>>,
}

lazy_static! {
    static ref GLOBAL_DEFAULT_TEXTURES: Arc<Mutex<DefaultTextures>> =
        Arc::new(Mutex::new(DefaultTextures::new()));
}

impl DefaultTextures {
    pub fn new() -> DefaultTextures {
        DefaultTextures {
            black: None,
            white: None,
        }
    }

    pub fn default() -> Arc<Mutex<DefaultTextures>> {
        GLOBAL_DEFAULT_TEXTURES.clone()
    }

    pub fn init(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.black = Some(Arc::new(util::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &wgpu::Color::BLACK,
        )));
        self.white = Some(Arc::new(util::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &wgpu::Color::WHITE,
        )));
    }

    pub fn get_black_texture(&self) -> Arc<wgpu::Texture> {
        match &self.black {
            Some(x) => x.clone(),
            None => panic!(),
        }
    }

    pub fn get_white_texture(&self) -> Arc<wgpu::Texture> {
        match &self.white {
            Some(x) => x.clone(),
            None => panic!(),
        }
    }

    pub fn get_black_texture_view(&self) -> wgpu::TextureView {
        self.get_black_texture()
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn get_white_texture_view(&self) -> wgpu::TextureView {
        self.get_white_texture()
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
