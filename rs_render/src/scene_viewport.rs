use crate::{antialias_type::EAntialiasType, command::Viewport};

#[derive(Clone)]
pub struct SceneViewport {
    pub scissor_rect: Option<glam::UVec4>,
    pub viewport: Option<Viewport>,
    pub anti_type: EAntialiasType,
}

impl SceneViewport {
    pub fn new() -> SceneViewport {
        SceneViewport {
            scissor_rect: None,
            viewport: None,
            anti_type: EAntialiasType::None,
        }
    }
}
