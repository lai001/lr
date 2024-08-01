pub mod attachment;
pub mod brdf_lut;
pub mod depth;
pub mod format_conversion;
pub mod fxaa;
pub mod global_shader;
pub mod grid;
pub mod irradiance_cube_map;
pub mod jfa;
pub mod jfa_composition;
pub mod mesh_view;
pub mod mesh_view_multiple_draw;
pub mod panorama_to_cube;
pub mod particle;
pub mod pre_filter_environment_cube_map;
pub mod primitive;
pub mod sdf2d_preprocess;
pub mod shading;
pub mod skeleton_shading;
pub mod virtual_texture_clean;
pub mod virtual_texture_feed_back;

use self::global_shader::GlobalShader;
use crate::global_shaders::{
    attachment::AttachmentShader,
    brdf_lut::BrdfLutShader,
    depth::{DepthShader, DepthSkinShader},
    format_conversion::Depth32FloatConvertRGBA8UnormShader,
    grid::GridShader,
    irradiance_cube_map::IrradianceCubeMapShader,
    jfa::JFAShader,
    jfa_composition::JFACompositionShader,
    mesh_view::MeshViewShader,
    mesh_view_multiple_draw::MeshViewMultipleDrawShader,
    panorama_to_cube::PanoramaToCubeShader,
    pre_filter_environment_cube_map::PreFilterEnvironmentCubeMapShader,
    sdf2d_preprocess::Sdf2dPreprocessShader,
    shading::ShadingShader,
    skeleton_shading::SkeletonShadingShader,
    virtual_texture_clean::VirtualTextureCleanShader,
    virtual_texture_feed_back::{
        SkinMeshVirtualTextureFeedBackShader, StaticMeshVirtualTextureFeedBackShader,
    },
};
use fxaa::FXAAShader;
use particle::ParticleShader;
use primitive::PrimitiveShader;

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
        Box::new(Sdf2dPreprocessShader {}),
        Box::new(JFAShader {}),
        Box::new(JFACompositionShader {}),
        Box::new(MeshViewShader {}),
        Box::new(MeshViewMultipleDrawShader {}),
        Box::new(DepthShader {}),
        Box::new(DepthSkinShader {}),
        Box::new(Depth32FloatConvertRGBA8UnormShader {}),
        Box::new(FXAAShader {}),
        Box::new(ParticleShader {}),
        Box::new(PrimitiveShader {}),
    ]
}
