use russimp_sys::*;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum TextureType {
    None = aiTextureType_aiTextureType_NONE as _,
    Diffuse = aiTextureType_aiTextureType_DIFFUSE as _,
    Specular = aiTextureType_aiTextureType_SPECULAR as _,
    Ambient = aiTextureType_aiTextureType_AMBIENT as _,
    Emissive = aiTextureType_aiTextureType_EMISSIVE as _,
    Height = aiTextureType_aiTextureType_HEIGHT as _,
    Normals = aiTextureType_aiTextureType_NORMALS as _,
    Shininess = aiTextureType_aiTextureType_SHININESS as _,
    Opacity = aiTextureType_aiTextureType_OPACITY as _,
    Displacement = aiTextureType_aiTextureType_DISPLACEMENT as _,
    LightMap = aiTextureType_aiTextureType_LIGHTMAP as _,
    Reflection = aiTextureType_aiTextureType_REFLECTION as _,
    BaseColor = aiTextureType_aiTextureType_BASE_COLOR as _,
    NormalCamera = aiTextureType_aiTextureType_NORMAL_CAMERA as _,
    EmissionColor = aiTextureType_aiTextureType_EMISSION_COLOR as _,
    Metalness = aiTextureType_aiTextureType_METALNESS as _,
    Roughness = aiTextureType_aiTextureType_DIFFUSE_ROUGHNESS as _,
    AmbientOcclusion = aiTextureType_aiTextureType_AMBIENT_OCCLUSION as _,
    Unknown = aiTextureType_aiTextureType_UNKNOWN as _,
    Sheen = aiTextureType_aiTextureType_SHEEN as _,
    ClearCoat = aiTextureType_aiTextureType_CLEARCOAT as _,
    Transmission = aiTextureType_aiTextureType_TRANSMISSION as _,
    Force32bit = aiTextureType__aiTextureType_Force32Bit as _,
}
