pub mod attachment;
pub mod blit;
pub mod box_culling;
pub mod brdf_lut;
pub mod depth;
pub mod format_conversion;
pub mod fxaa;
pub mod global_shader;
pub mod grid;
pub mod irradiance_cube_map;
pub mod jfa;
pub mod jfa_composition;
pub mod light_culling;
pub mod mesh_view;
pub mod mesh_view_multiple_draw;
pub mod panorama_to_cube;
pub mod particle;
pub mod pre_filter_environment_cube_map;
pub mod primitive;
pub mod sdf2d_preprocess;
pub mod shading;
pub mod skeleton_shading;
pub mod view_depth;
pub mod virtual_texture_clean;
pub mod virtual_texture_feed_back;

use self::global_shader::GlobalShader;
use attachment::AttachmentShader;
use blit::BlitShader;
use box_culling::BoxCullingShader;
use brdf_lut::BrdfLutShader;
use depth::{DepthShader, DepthSkinShader};
use format_conversion::Depth32FloatConvertRGBA8UnormShader;
use fxaa::FXAAShader;
use grid::GridShader;
use irradiance_cube_map::IrradianceCubeMapShader;
use jfa::JFAShader;
use jfa_composition::JFACompositionShader;
use light_culling::LightCullingShader;
use mesh_view::MeshViewShader;
use mesh_view_multiple_draw::MeshViewMultipleDrawShader;
use panorama_to_cube::PanoramaToCubeShader;
use particle::ParticleShader;
use pre_filter_environment_cube_map::PreFilterEnvironmentCubeMapShader;
use primitive::PrimitiveShader;
use sdf2d_preprocess::Sdf2dPreprocessShader;
use shading::ShadingShader;
use skeleton_shading::SkeletonShadingShader;
use view_depth::ViewDepthShader;
use virtual_texture_clean::VirtualTextureCleanShader;
use virtual_texture_feed_back::{
    SkinMeshVirtualTextureFeedBackShader, StaticMeshVirtualTextureFeedBackShader,
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
        Box::new(LightCullingShader {}),
        Box::new(ViewDepthShader {}),
        Box::new(BoxCullingShader {}),
        Box::new(BlitShader {}),
    ]
}
