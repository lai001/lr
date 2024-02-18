use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VirtualTextureSetting {
    pub is_enable: bool,
    pub tile_size: u32,
    pub physical_texture_size: u32,
    pub virtual_texture_size: u32,
    pub feed_back_texture_div: u32,
    pub mipmap_level_bias: f32,
    pub mipmap_level_scale: f32,
    pub feedback_bias: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderSettings {
    pub virtual_texture_setting: VirtualTextureSetting,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub render_setting: RenderSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_setting: RenderSettings {
                virtual_texture_setting: VirtualTextureSetting {
                    tile_size: 256,
                    physical_texture_size: 4096,
                    virtual_texture_size: 512 * 1000,
                    feed_back_texture_div: 10,
                    mipmap_level_bias: 0.0,
                    mipmap_level_scale: 0.0,
                    feedback_bias: 0.0,
                    is_enable: true,
                },
            },
        }
    }
}
