pub mod attachment;
pub mod brdf_lut;
pub mod global_shader;
pub mod grid;
pub mod irradiance_cube_map;
pub mod panorama_to_cube;
pub mod pre_filter_environment_cube_map;
pub mod shading;
pub mod skeleton_shading;
pub mod virtual_texture_clean;
pub mod virtual_texture_feed_back;

use self::global_shader::GlobalShader;
use crate::global_shaders::{
    attachment::AttachmentShader,
    brdf_lut::BrdfLutShader,
    grid::GridShader,
    irradiance_cube_map::IrradianceCubeMapShader,
    panorama_to_cube::PanoramaToCubeShader,
    pre_filter_environment_cube_map::PreFilterEnvironmentCubeMapShader,
    shading::ShadingShader,
    skeleton_shading::SkeletonShadingShader,
    virtual_texture_clean::VirtualTextureCleanShader,
    virtual_texture_feed_back::{
        SkinMeshVirtualTextureFeedBackShader, StaticMeshVirtualTextureFeedBackShader,
    },
};

pub fn get_buildin_shaders() -> Vec<Box<dyn GlobalShader>> {
    vec![
        Box::new(AttachmentShader {}),
        Box::new(PanoramaToCubeShader {}),
        Box::new(BrdfLutShader {}),
        Box::new(IrradianceCubeMapShader {}),
        Box::new(PreFilterEnvironmentCubeMapShader {}),
        Box::new(ShadingShader {}),
        Box::new(SkeletonShadingShader {}),
        Box::new(GridShader {}),
        Box::new(StaticMeshVirtualTextureFeedBackShader {}),
        Box::new(SkinMeshVirtualTextureFeedBackShader {}),
        Box::new(VirtualTextureCleanShader {}),
    ]
}
