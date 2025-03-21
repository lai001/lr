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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum PowerPreference {
    #[default]
    None,
    LowPower,
    HighPerformance,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum Backends {
    #[default]
    Primary,
    Vulkan,
    GL,
    DX12,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum EAntialiasType {
    #[default]
    None,
    FXAA,
    MSAA,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderSettings {
    pub power_preference: PowerPreference,
    pub backends: Backends,
    pub android_backends: Backends,
    pub virtual_texture_setting: VirtualTextureSetting,
    #[serde(default)]
    pub antialias_type: EAntialiasType,
    pub is_enable_multithread_rendering: bool,
    pub is_enable_debugging: bool,
    pub is_enable_dump_material_shader_code: bool,
    pub is_enable_light_culling_acceleration: bool,
    pub is_enable_multiple_resolution_mesh: bool,
}

impl RenderSettings {
    pub fn get_backends_platform(&self) -> Backends {
        #[cfg(not(target_os = "android"))]
        return self.backends.clone();
        #[cfg(target_os = "android")]
        return self.android_backends.clone();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditorSettings {
    pub is_auto_open_last_project: bool,
    pub is_enable_log_to_file: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub editor_settings: EditorSettings,
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
                power_preference: PowerPreference::HighPerformance,
                #[cfg(target_os = "windows")]
                backends: Backends::DX12,
                #[cfg(not(target_os = "windows"))]
                backends: Backends::Primary,
                android_backends: Backends::Primary,
                antialias_type: EAntialiasType::None,
                is_enable_multithread_rendering: false,
                is_enable_debugging: true,
                is_enable_dump_material_shader_code: true,
                is_enable_light_culling_acceleration: false,
                is_enable_multiple_resolution_mesh: false,
            },
            editor_settings: EditorSettings {
                is_auto_open_last_project: true,
                is_enable_log_to_file: false,
            },
        }
    }
}
