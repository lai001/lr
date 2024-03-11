use lazy_static::lazy_static;

lazy_static! {
    pub static ref AI_CONFIG_GLOB_MEASURE_TIME: &'static str =
        std::str::from_utf8(russimp_sys::AI_CONFIG_GLOB_MEASURE_TIME).unwrap();
    pub static ref AI_CONFIG_IMPORT_NO_SKELETON_MESHES: &'static str =
        std::str::from_utf8(russimp_sys::AI_CONFIG_IMPORT_NO_SKELETON_MESHES).unwrap();
    pub static ref AI_CONFIG_PP_CT_MAX_SMOOTHING_ANGLE: &'static str =
        std::str::from_utf8(russimp_sys::AI_CONFIG_PP_CT_MAX_SMOOTHING_ANGLE).unwrap();
    pub static ref AI_CONFIG_FBX_USE_SKELETON_BONE_CONTAINER: &'static str =
        std::str::from_utf8(russimp_sys::AI_CONFIG_FBX_USE_SKELETON_BONE_CONTAINER).unwrap();
}
