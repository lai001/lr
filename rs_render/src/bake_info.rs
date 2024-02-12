#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BakeInfo {
    pub is_bake_environment: bool,
    pub is_bake_irradiance: bool,
    pub is_bake_brdflut: bool,
    pub is_bake_pre_filter: bool,
    pub environment_cube_map_length: u32,
    pub irradiance_cube_map_length: u32,
    pub irradiance_sample_count: u32,
    pub pre_filter_cube_map_length: u32,
    pub pre_filter_cube_map_max_mipmap_level: u32,
    pub pre_filter_sample_count: u32,
    pub brdflutmap_length: u32,
    pub brdf_sample_count: u32,
}

impl Default for BakeInfo {
    fn default() -> Self {
        Self {
            is_bake_environment: true,
            is_bake_irradiance: true,
            is_bake_brdflut: true,
            is_bake_pre_filter: true,
            environment_cube_map_length: 512,
            irradiance_cube_map_length: 512,
            irradiance_sample_count: 1024,
            pre_filter_cube_map_length: 512,
            pre_filter_cube_map_max_mipmap_level: 11,
            pre_filter_sample_count: 1024,
            brdflutmap_length: 512,
            brdf_sample_count: 1024,
        }
    }
}
