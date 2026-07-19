use crate::command::TextureHandle;

#[derive(Clone)]
pub enum UICanvasType {
    FrameBuffer(TextureHandle),
    Window(isize),
}
