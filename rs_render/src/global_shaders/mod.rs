pub mod attachment;
pub mod brdf_lut;
pub mod global_shader;
pub mod irradiance_cube_map;
pub mod panorama_to_cube;
pub mod phong;
pub mod pre_filter_environment_cube_map;

use self::global_shader::GlobalShader;
use crate::global_shaders::{
    attachment::AttachmentShader, brdf_lut::BrdfLutShader,
    irradiance_cube_map::IrradianceCubeMapShader, panorama_to_cube::PanoramaToCubeShader,
    phong::PhongShader, pre_filter_environment_cube_map::PreFilterEnvironmentCubeMapShader,
};

pub fn get_buildin_shaders() -> Vec<Box<dyn GlobalShader>> {
    vec![
        Box::new(PhongShader {}),
        Box::new(AttachmentShader {}),
        Box::new(PanoramaToCubeShader {}),
        Box::new(BrdfLutShader {}),
        Box::new(IrradianceCubeMapShader {}),
        Box::new(PreFilterEnvironmentCubeMapShader {}),
    ]
}
