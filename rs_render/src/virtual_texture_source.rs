use std::sync::{Arc, Mutex};

pub trait TVirtualTextureSource: Send + Sync {
    fn get_tile_image(&mut self, index: &glam::UVec3) -> Option<image::DynamicImage>;
    fn get_size(&self) -> glam::UVec2;
}

pub struct VirtualTextureSource {
    inner: Arc<Mutex<Box<dyn TVirtualTextureSource>>>,
}

impl VirtualTextureSource {
    pub fn new(source: Arc<Mutex<Box<dyn TVirtualTextureSource>>>) -> Self {
        Self { inner: source }
    }

    pub fn get_tile_image(&mut self, index: &glam::UVec3) -> Option<image::DynamicImage> {
        self.inner.lock().unwrap().get_tile_image(index)
    }

    pub fn get_size(&self) -> glam::UVec2 {
        self.inner.lock().unwrap().get_size()
    }
}
