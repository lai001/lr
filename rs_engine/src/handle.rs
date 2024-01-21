use rs_foundation::id_generator::IDGenerator;
use std::{ops::Deref, rc::Rc};

pub struct HandleManager {
    texture_idgenerator: IDGenerator,
    buffer_idgenerator: IDGenerator,
    gui_texture_idgenerator: IDGenerator,
}

impl HandleManager {
    pub fn new() -> HandleManager {
        HandleManager {
            texture_idgenerator: IDGenerator::new(),
            buffer_idgenerator: IDGenerator::new(),
            gui_texture_idgenerator: IDGenerator::new(),
        }
    }

    pub fn next_texture(&mut self) -> TextureHandle {
        let new_id = self.texture_idgenerator.get_next_id();
        TextureHandle {
            inner: Rc::new(new_id),
        }
    }

    pub fn next_ui_texture(&mut self) -> EGUITextureHandle {
        let new_id = self.gui_texture_idgenerator.get_next_id();
        EGUITextureHandle {
            inner: Rc::new(new_id),
        }
    }

    pub fn next_buffer(&mut self) -> BufferHandle {
        let new_id = self.buffer_idgenerator.get_next_id();
        BufferHandle {
            inner: Rc::new(new_id),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct TextureHandle {
    inner: Rc<u64>,
}

impl Deref for TextureHandle {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct EGUITextureHandle {
    inner: Rc<u64>,
}

impl Deref for EGUITextureHandle {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct BufferHandle {
    inner: Rc<u64>,
}

impl Deref for BufferHandle {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
