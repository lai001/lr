use crate::util;
use std::sync::{Arc, Mutex};

pub struct DefaultTextures {
    black: Arc<Option<wgpu::Texture>>,
    white: Arc<Option<wgpu::Texture>>,
    normal_texture: Arc<Option<wgpu::Texture>>,
}

lazy_static! {
    static ref GLOBAL_DEFAULT_TEXTURES: Arc<Mutex<DefaultTextures>> =
        Arc::new(Mutex::new(DefaultTextures::new()));
}

impl DefaultTextures {
    pub fn new() -> DefaultTextures {
        DefaultTextures {
            black: Arc::new(None),
            white: Arc::new(None),
            normal_texture: Arc::new(None),
        }
    }

    pub fn default() -> Arc<Mutex<DefaultTextures>> {
        GLOBAL_DEFAULT_TEXTURES.clone()
    }

    pub fn init(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.black = Arc::new(Some(util::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &wgpu::Color::BLACK,
        )));
        self.white = Arc::new(Some(util::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &wgpu::Color::WHITE,
        )));
        self.normal_texture = Arc::new(Some(util::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &wgpu::Color {
                r: 0.5,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
        )));
    }

    pub fn get_black_texture(&self) -> Arc<Option<wgpu::Texture>> {
        self.black.clone()
    }

    pub fn get_white_texture(&self) -> Arc<Option<wgpu::Texture>> {
        self.white.clone()
    }

    pub fn get_normal_texture(&self) -> Arc<Option<wgpu::Texture>> {
        self.normal_texture.clone()
    }

    pub fn get_black_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref black) = *self.black {
            return black.create_view(&wgpu::TextureViewDescriptor::default());
        } else {
            panic!()
        }
    }

    pub fn get_white_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref white) = *self.white {
            return white.create_view(&wgpu::TextureViewDescriptor::default());
        } else {
            panic!()
        }
    }

    pub fn get_normal_texture_view(&self) -> wgpu::TextureView {
        if let Some(ref nomal_texture) = *self.normal_texture {
            return nomal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        } else {
            panic!()
        }
    }
}
