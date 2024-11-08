#include "constants.wgsl"
#include "common.wgsl"
#include "global_constants.wgsl"
#include "virtual_texture.wgsl"
#include "light.wgsl"

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord0: vec2<f32>,
    @location(2) vertex_color: vec4<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) tangent: vec3<f32>,
    @location(5) bitangent: vec3<f32>,
#ifdef SKELETON_MAX_BONES
    @location(6) bone_ids: vec4<i32>,
    @location(7) bone_weights: vec4<f32>,
#endif
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

struct Constants {
    model: mat4x4<f32>,
    id: u32,
};

#ifdef SKELETON_MAX_BONES
struct SkinConstants {
    bones: array<mat4x4<f32>, SKELETON_MAX_BONES>,
};
#endif

#ifdef VIRTUAL_TEXTURE
struct VirtualTextureConstants {
    virtual_texture_size: vec2<f32>,
    virtual_texture_max_lod: u32,
};
#endif

struct UserAttributes {
    base_color: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32,
    metallic: f32,
    opacity: f32,
    clear_coat: f32,
    clear_coat_roughness: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord0: vec2<f32>,
    @location(3) vertex_color: vec4<f32>,
    @location(4) tbn_t: vec3<f32>,
    @location(5) tbn_b: vec3<f32>,
    @location(6) tbn_n: vec3<f32>,
    @location(7) frag_position_at_light_space: vec4<f32>,
};

struct ClearCoatInfo {
    attenuation: f32,
    specular: vec3<f32>,
};

struct ShadingInfo {
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    base_color: vec3<f32>,
    shading_reflected: vec3<f32>,
    metallic: f32,
    roughness: f32,
    opacity: f32,
    nov: f32,
    noh: f32,
    f0: vec3<f32>,
    clear_coat_info: ClearCoatInfo,
};

GROUP_BINDING(GLOBAL_CONSTANTS) var<uniform> global_constants: GlobalConstants;

GROUP_BINDING(BASE_COLOR_SAMPLER) var base_color_sampler: sampler;

GROUP_BINDING(PHYSICAL_TEXTURE) var physical_texture: texture_2d<f32>;

GROUP_BINDING(PAGE_TABLE_TEXTURE) var page_table_texture: texture_2d<u32>;

GROUP_BINDING(BRDFLUT_TEXTURE) var brdflut_texture: texture_2d<f32>;

GROUP_BINDING(PRE_FILTER_CUBE_MAP_TEXTURE) var pre_filter_cube_map_texture: texture_cube<f32>;

GROUP_BINDING(IRRADIANCE_TEXTURE) var irradiance_texture: texture_cube<f32>;

GROUP_BINDING(SHADOW_MAP) var shadow_map: texture_depth_2d;

GROUP_BINDING(CONSTANTS) var<uniform> constants: Constants;

GROUP_BINDING(POINT_LIGHTS) var<uniform> point_lights: PointLights;

GROUP_BINDING(SPOT_LIGHTS) var<uniform> spot_lights: SpotLights;

#ifdef SKELETON_MAX_BONES
GROUP_BINDING(SKIN_CONSTANTS) var<uniform> skin_constants: SkinConstants;
#endif

#ifdef VIRTUAL_TEXTURE
GROUP_BINDING(VIRTUAL_TEXTURE_CONSTANTS) var<uniform> virtual_texture_constants: VirtualTextureConstants;
#endif

#ifdef USER_TEXTURES
    USER_TEXTURES
#endif

#ifdef MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS
    MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS
#endif

fn shadow_calculation(shadow_map: texture_depth_2d, frag_position_at_light_space: vec4<f32>) -> f32 {
    var bias = 0.005;
    var proj_coords: vec3<f32> = frag_position_at_light_space.xyz / frag_position_at_light_space.w;
    var proj_coords_uv = proj_coords * 0.5 + 0.5;
    proj_coords_uv.y = 1.0 - proj_coords_uv.y;
    var closest_depth = textureSample(shadow_map, base_color_sampler, proj_coords_uv.xy);
    var current_depth = proj_coords.z - bias;
    var shadow = 0.0;
    if (current_depth > closest_depth) {
        shadow = 1.0;
    }
    return shadow;
}

fn D(N: vec3<f32>, H: vec3<f32>, a: f32) -> f32 {
    var a2 = a * a;
    var NdotH = max(dot(N, H), 0.0);
    var NdotH2 = NdotH * NdotH;

    var nom = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

fn cal_f0(ior: f32) -> f32 {
    return pow((ior - 1.5)  / (ior + 1.5), 2.0);
}

fn D_GGX(NoH: f32, a: f32) -> f32 {
    var a2 = a * a;
    var f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

fn V_Kelemen(LoH: f32) -> f32 {
    return 0.25 / (LoH * LoH);
}

fn F(H: vec3<f32>, V: vec3<f32>, F0: f32) -> f32 {
    var cosTheta = dot(H, V);
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

fn F3(H: vec3<f32>, V: vec3<f32>, F0: vec3<f32>) -> vec3<f32> {
    var cosTheta = dot(H, V);
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

fn F_Schlick(f0: f32, f90: f32, VoH: f32) -> f32 {
    var a = 1.0 - VoH;
    return f0 + (f90 - f0) * (a*a*a*a*a);
}

fn SubG(InAngle: f32, k: f32) -> f32 {
    var nom = InAngle;
    var denom = InAngle * (1.0 - k) + k;
    return nom / denom;
}

fn G(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, k: f32) -> f32 {
    var NdotV = max(dot(N, V), 0.0);
    var NdotL = max(dot(N, L), 0.0);
    var ggx1 = SubG(NdotV, k);
    var ggx2 = SubG(NdotL, k);
    return ggx1 * ggx2;
}

fn ibl_diffuse_color(shading_info: ShadingInfo, irradiance_texture: texture_cube<f32>) -> vec3<f32> {
    var clear_coat_info = shading_info.clear_coat_info;
    var irradiance = textureSample(irradiance_texture, base_color_sampler, shading_info.normal).xyz;
    var diffuse_color = shading_info.base_color.rgb * irradiance;
    return diffuse_color * clear_coat_info.attenuation;
}

fn ibl_specular_color(shading_info: ShadingInfo,
    light_reflection_vec: vec3<f32>,
    pre_filter_cube_map_texture: texture_cube<f32>,
    brdflut_texture: texture_2d<f32>) -> vec3<f32>
{
    var clear_coat_info = shading_info.clear_coat_info;
    var levels = f32(textureNumLevels(pre_filter_cube_map_texture)) - 1.0;
    var lod = shading_info.roughness * levels;
    var pre_filter_value = textureSampleLevel(pre_filter_cube_map_texture, base_color_sampler, light_reflection_vec, lod).xyz;
    var brdf_value = textureSample(brdflut_texture, base_color_sampler, vec2<f32>(shading_info.nov, shading_info.roughness)).xy;
    var specular_color = (shading_info.f0 * brdf_value.x + brdf_value.y) * pre_filter_value;
    return specular_color * clear_coat_info.attenuation + clear_coat_info.specular;
}

fn ibl_light(shading_info: ShadingInfo,
    irradiance_texture: texture_cube<f32>,
    pre_filter_cube_map_texture: texture_cube<f32>,
    brdflut_texture: texture_2d<f32>) -> vec3<f32>
{
    var diffuse_color = ibl_diffuse_color(shading_info, irradiance_texture);
    var specular_color = ibl_specular_color(shading_info, shading_info.shading_reflected, pre_filter_cube_map_texture, brdflut_texture);
    return diffuse_color + specular_color;
}

fn fetch_clear_coat_info(
    pre_filter_cube_map_texture: texture_cube<f32>,
    nov: f32,
    shading_reflected: vec3<f32>,
    clear_coat: f32,
    clear_coat_roughness: f32) -> ClearCoatInfo
{
    var Fc = F_Schlick(0.04, 1.0, nov) * clear_coat;
    var attenuation = 1.0 - Fc;
    var levels = f32(textureNumLevels(pre_filter_cube_map_texture)) - 1.0;
    var lod = levels * clear_coat_roughness * (2.0 - clear_coat_roughness);
    var pre_filter_value = textureSampleLevel(pre_filter_cube_map_texture, base_color_sampler, shading_reflected, lod).xyz;
    var specular = pre_filter_value * Fc;
    var info: ClearCoatInfo;
    info.attenuation = attenuation;
    info.specular = specular;
    return info;
}

fn get_normal(i_normal: vec3<f32>, tbn: mat3x3<f32>) -> vec3<f32> {
    var normal = normalize(i_normal * 2.0 - 1.0);
    var normal_w = normalize(tbn * normal);
    return normal_w;
}

fn get_user_attributes(vertex_output: VertexOutput) -> UserAttributes {
    var user_attributes: UserAttributes;
#ifdef MATERIAL_SHADER_CODE
    MATERIAL_SHADER_CODE
#endif
    return user_attributes;
}

fn get_shading_info(user_attributes: UserAttributes, vertex_output: VertexOutput) -> ShadingInfo {
    var tbn = mat3x3<f32>(
        vertex_output.tbn_t,
        vertex_output.tbn_b,
        vertex_output.tbn_n,
    );
    var view_direction = normalize(global_constants.view_position - vertex_output.frag_position.xyz);
    var normal_world_space = get_normal(user_attributes.normal, tbn);
    var nov = max(dot(normal_world_space, view_direction), 0.0001);
    var shading_info: ShadingInfo;
    shading_info.base_color = user_attributes.base_color;
    shading_info.metallic = user_attributes.metallic;
    shading_info.roughness = user_attributes.roughness;
    shading_info.normal = normal_world_space;
    shading_info.view_direction = view_direction;
    shading_info.f0 = mix(vec3<f32>(1.0, 1.0, 1.0) * 0.04, user_attributes.base_color.xyz, user_attributes.metallic);
    shading_info.nov = nov;
    shading_info.opacity = user_attributes.opacity;
    shading_info.shading_reflected = reflect(view_direction, normal_world_space);

    shading_info.clear_coat_info = fetch_clear_coat_info(pre_filter_cube_map_texture,
        nov,
        shading_info.shading_reflected,
        user_attributes.clear_coat,
        user_attributes.clear_coat_roughness);
    return shading_info;
}

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
#ifdef SKELETON_MAX_BONES
    var bone_transform = skin_constants.bones[vertex_in.bone_ids[0]] * vertex_in.bone_weights[0];
    bone_transform += skin_constants.bones[vertex_in.bone_ids[1]] * vertex_in.bone_weights[1];
    bone_transform += skin_constants.bones[vertex_in.bone_ids[2]] * vertex_in.bone_weights[2];
    bone_transform += skin_constants.bones[vertex_in.bone_ids[3]] * vertex_in.bone_weights[3];
#endif
    let mvp = global_constants.view_projection * constants.model;
    var vertex_output: VertexOutput;
    vertex_output.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    vertex_output.tex_coord0 = vertex_in.tex_coord0;
    vertex_output.vertex_color = vertex_in.vertex_color;
    vertex_output.frag_position = (constants.model * vec4<f32>(vertex_in.position, 1.0)).xyz;

#ifdef SKELETON_MAX_BONES
    vertex_output.position = mvp * bone_transform * vec4<f32>(vertex_in.position, 1.0);
    vertex_output.frag_position = (constants.model * bone_transform * vec4<f32>(vertex_in.position, 1.0)).xyz;
    vertex_output.normal = (transpose(inverse(constants.model * bone_transform)) * vec4<f32>(vertex_in.normal, 0.0)).xyz;
#else
    vertex_output.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    vertex_output.frag_position = (constants.model * vec4<f32>(vertex_in.position, 1.0)).xyz;
    vertex_output.normal = vertex_in.normal;
#endif

    vertex_output.tbn_t = (constants.model * vec4<f32>(vertex_in.tangent, 0.0)).xyz;
    vertex_output.tbn_b = (constants.model * vec4<f32>(vertex_in.bitangent, 0.0)).xyz;
    vertex_output.tbn_n = (constants.model * vec4<f32>(vertex_output.normal, 0.0)).xyz;

    vertex_output.frag_position_at_light_space = global_constants.light_space_matrix * vec4<f32>(vertex_output.frag_position, 1.0);

    return vertex_output;
}

@fragment fn fs_main(vertex_output: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;

    var user_attributes: UserAttributes = get_user_attributes(vertex_output);
    var shading_info = get_shading_info(user_attributes, vertex_output);

    var ibl_color = ibl_light(shading_info, irradiance_texture, pre_filter_cube_map_texture, brdflut_texture);

    var shadow = shadow_calculation(shadow_map, vertex_output.frag_position_at_light_space);

    var point_light_color: vec3<f32>;
    for (var i = 0u; i < MAX_POINT_LIGHTS_NUM ; i++) {
        if (i >= point_lights.available) {
            break;
        }
        point_light_color = point_light_color + point_light(point_lights.lights[i], shading_info.normal, vertex_output.frag_position, shading_info.view_direction);
    }

    var spot_light_color: vec3<f32>;
    for (var i = 0u; i < MAX_SPOT_LIGHTS_NUM ; i++) {
        if (i >= spot_lights.available) {
            break;
        }
        spot_light_color = spot_light_color + spot_light(spot_lights.lights[i], shading_info.normal, vertex_output.frag_position, shading_info.view_direction);
    }

    var light_color = ibl_color + point_light_color + spot_light_color;
    fragment_output.color = vec4<f32>(mix(light_color, light_color * vec3<f32>(0.5), shadow), 1.0);

    // fragment_output.color = vec4<f32>(ibl_color, 1.0);
    if (global_constants.debug_shading == DEBUG_SHADING_TYPE_BASE_COLOR) {
        fragment_output.color = vec4<f32>(shading_info.base_color, 1.0);
    } else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_METALLIC) {
        fragment_output.color = vec4<f32>(vec3<f32>(shading_info.metallic), 1.0);
    } else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_ROUGHNESS) {
        fragment_output.color = vec4<f32>(vec3<f32>(shading_info.roughness), 1.0);
    } else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_NORMAL) {
        fragment_output.color = vec4<f32>((shading_info.normal + 1.0) / 2.0, 1.0);
    } else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_VERTEX_COLOR_0) {
        fragment_output.color = vertex_output.vertex_color;
    } else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_SHADOW) {
        fragment_output.color = vec4<f32>(vec3<f32>(mix(1.0, 0.0, shadow)), 1.0);
    }
    return fragment_output;
}