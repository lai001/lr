#include "constants.slang"
#include "common.slang"
#include "global_constants.slang"
#include "virtual_texture.slang"
#include "light.slang"
#include "cluster_light.slang"

struct VertexIn
{
    [[vk::location(0)]] float3 position;
    [[vk::location(1)]] float2 tex_coord0;
    [[vk::location(2)]] float4 vertex_color;
    [[vk::location(3)]] float3 normal;
    [[vk::location(4)]] float3 tangent;
    [[vk::location(5)]] float3 bitangent;
#ifdef SKELETON_MAX_BONES
    [[vk::location(6)]] int4 bone_ids;
    [[vk::location(7)]] float4 bone_weights;
#endif
};

struct FragmentOutput
{
    [[vk::location(0)]] float4 color : SV_Target0;
};

struct Constants
{
    float4x4 model;
    uint id;
};

#ifdef SKELETON_MAX_BONES
struct SkinConstants
{
    float4x4 bones[SKELETON_MAX_BONES];
};
#endif

#ifdef VIRTUAL_TEXTURE
struct VirtualTextureConstants
{
    float2 virtual_texture_size;
    uint virtual_texture_max_lod;
};
#endif

struct UserAttributes
{
    float3 base_color;
    float3 normal;
    float roughness;
    float metallic;
    float opacity;
    float clear_coat;
    float clear_coat_roughness;
};

struct VertexOutput
{
    float4 position : SV_Position;
    [[vk::location(0)]] float3 frag_position;
    [[vk::location(1)]] float3 normal;
    [[vk::location(2)]] float2 tex_coord0;
    [[vk::location(3)]] float4 vertex_color;
    [[vk::location(4)]] float3 tbn_t;
    [[vk::location(5)]] float3 tbn_b;
    [[vk::location(6)]] float3 tbn_n;
    [[vk::location(7)]] float4 frag_position_at_light_space;
};

struct ClearCoatInfo
{
    float attenuation;
    float3 specular;
};

struct ShadingInfo
{
    float3 normal;
    float3 view_direction;
    float3 base_color;
    float3 shading_reflected;
    float metallic;
    float roughness;
    float opacity;
    float nov;
    float noh;
    float3 f0;
    ClearCoatInfo clear_coat_info;
};

GROUP_BINDING(GLOBAL_CONSTANTS) ConstantBuffer<GlobalConstants> global_constants;

GROUP_BINDING(BASE_COLOR_SAMPLER) SamplerState base_color_sampler;

GROUP_BINDING(PHYSICAL_TEXTURE) Texture2D physical_texture;

GROUP_BINDING(PAGE_TABLE_TEXTURE) Texture2D<uint4> page_table_texture;

GROUP_BINDING(BRDFLUT_TEXTURE) Texture2D brdflut_texture;

GROUP_BINDING(PRE_FILTER_CUBE_MAP_TEXTURE) TextureCube pre_filter_cube_map_texture;

GROUP_BINDING(IRRADIANCE_TEXTURE) TextureCube irradiance_texture;

GROUP_BINDING(SHADOW_MAP) DepthTexture2D shadow_map;

GROUP_BINDING(CONSTANTS) ConstantBuffer<Constants> constants;

GROUP_BINDING(SPOT_LIGHTS) ConstantBuffer<SpotLights> spot_lights;

#ifdef SUPPORT_CLUSTER_LIGHTS
GROUP_BINDING(POINT_LIGHTS) StructuredBuffer<PointLight> point_lights;
GROUP_BINDING(CLUSTER_LIGHT) StructuredBuffer<uint> cluster_lights;
GROUP_BINDING(CLUSTER_LIGHT_INDEX) StructuredBuffer<ClusterLightIndex> cluster_light_indices;
#else
GROUP_BINDING(POINT_LIGHTS) ConstantBuffer<PointLights> point_lights;
#endif

#ifdef SKELETON_MAX_BONES
GROUP_BINDING(SKIN_CONSTANTS) ConstantBuffer<SkinConstants> skin_constants;
#endif

#ifdef VIRTUAL_TEXTURE
GROUP_BINDING(VIRTUAL_TEXTURE_CONSTANTS) ConstantBuffer<VirtualTextureConstants> virtual_texture_constants;
#endif

#ifdef USER_TEXTURES
    USER_TEXTURES
#endif

#ifdef MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS
    MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS
#endif

#ifdef USER_MATERIAL_PARAMENTERS_UNIFORM
    USER_MATERIAL_PARAMENTERS_UNIFORM
#endif

float shadow_calculation(float4 frag_position_at_light_space)
{
    float bias = 0.005;
    float3 proj_coords = frag_position_at_light_space.xyz / frag_position_at_light_space.w;
    float3 proj_coords_uv = proj_coords * 0.5 + 0.5;
    proj_coords_uv.y = 1.0 - proj_coords_uv.y;
    float closest_depth = shadow_map.Sample(base_color_sampler, proj_coords_uv.xy).r;
    float current_depth = proj_coords.z - bias;
    float shadow = 0.0;
    if (current_depth > closest_depth)
        shadow = 1.0;
    return shadow;
}

float D(float3 N, float3 H, float a)
{
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

float cal_f0(float ior)
{
    return pow((ior - 1.5) / (ior + 1.5), 2.0);
}

float D_GGX(float NoH, float a)
{
    float a2 = a * a;
    float f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

float V_Kelemen(float LoH)
{
    return 0.25 / (LoH * LoH);
}

float F(float3 H, float3 V, float F0_val)
{
    float cosTheta = dot(H, V);
    return F0_val + (1.0 - F0_val) * pow(1.0 - cosTheta, 5.0);
}

float3 F3(float3 H, float3 V, float3 F0_val)
{
    float cosTheta = dot(H, V);
    return F0_val + (1.0 - F0_val) * pow(1.0 - cosTheta, 5.0);
}

float F_Schlick(float f0, float f90, float VoH)
{
    float a = 1.0 - VoH;
    return f0 + (f90 - f0) * (a * a * a * a * a);
}

float SubG(float InAngle, float k)
{
    float nom = InAngle;
    float denom = InAngle * (1.0 - k) + k;
    return nom / denom;
}

float G(float3 N, float3 V, float3 L, float k)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx1 = SubG(NdotV, k);
    float ggx2 = SubG(NdotL, k);
    return ggx1 * ggx2;
}

float3 ibl_diffuse_color(ShadingInfo shading_info)
{
    ClearCoatInfo clear_coat_info = shading_info.clear_coat_info;
    float3 irradiance = irradiance_texture.Sample(base_color_sampler, shading_info.normal).rgb;
    float3 diffuse_color = shading_info.base_color.rgb * irradiance;
    return diffuse_color * clear_coat_info.attenuation;
}

float3 ibl_specular_color(ShadingInfo shading_info, float3 light_reflection_vec)
{
    ClearCoatInfo clear_coat_info = shading_info.clear_coat_info;
    uint width;
    uint height;
    uint levels;
    pre_filter_cube_map_texture.GetDimensions(0, width, height, levels);
    float lod = shading_info.roughness * float(levels - 1);
    float3 pre_filter_value = pre_filter_cube_map_texture.SampleLevel(base_color_sampler, light_reflection_vec, lod).rgb;
    float2 brdf_value = brdflut_texture.Sample(base_color_sampler, float2(shading_info.nov, shading_info.roughness)).rg;
    float3 specular_color = (shading_info.f0 * brdf_value.x + brdf_value.y) * pre_filter_value;
    return specular_color * clear_coat_info.attenuation + clear_coat_info.specular;
}

float3 ibl_light(ShadingInfo shading_info)
{
    float3 diffuse_color = ibl_diffuse_color(shading_info);
    float3 specular_color = ibl_specular_color(shading_info, shading_info.shading_reflected);
    return diffuse_color + specular_color;
}

ClearCoatInfo fetch_clear_coat_info(
    float nov,
    float3 shading_reflected,
    float clear_coat,
    float clear_coat_roughness)
{
    float Fc = F_Schlick(0.04, 1.0, nov) * clear_coat;
    float attenuation = 1.0 - Fc;
    uint width;
    uint height; 
    uint levels; 
    pre_filter_cube_map_texture.GetDimensions(0, width, height, levels);
    float lod = float(levels - 1) * clear_coat_roughness * (2.0 - clear_coat_roughness);
    float3 pre_filter_value = pre_filter_cube_map_texture.SampleLevel(base_color_sampler, shading_reflected, lod).rgb;
    float3 specular = pre_filter_value * Fc;

    ClearCoatInfo info;
    info.attenuation = attenuation;
    info.specular = specular;
    return info;
}

float3 get_normal(float3 i_normal, float3x3 tbn)
{
    float3 normal = normalize(i_normal * 2.0 - 1.0);
    float3 normal_w = normalize(mul(tbn, normal));
    return normal_w;
}

UserAttributes get_user_attributes(VertexOutput vertex_output)
{
    UserAttributes user_attributes;
    user_attributes.base_color = float3(0.0, 0.0, 0.0);
    user_attributes.normal = float3(0.5, 0.5, 1.0);
    user_attributes.roughness = 0.0;
    user_attributes.metallic = 0.0;
    user_attributes.opacity = 1.0;
    user_attributes.clear_coat = 0.0;
    user_attributes.clear_coat_roughness = 0.0;
    return user_attributes;
}

ShadingInfo get_shading_info(UserAttributes user_attributes, VertexOutput vertex_output)
{
    float3x3 tbn = float3x3(
        vertex_output.tbn_t,
        vertex_output.tbn_b,
        vertex_output.tbn_n);

    float3 view_direction = normalize(global_constants.view_position - vertex_output.frag_position.xyz);
    float3 normal_world_space = get_normal(user_attributes.normal, tbn);
    float nov = max(dot(normal_world_space, view_direction), 0.0001);

    ShadingInfo shading_info;
    shading_info.base_color = user_attributes.base_color;
    shading_info.metallic = user_attributes.metallic;
    shading_info.roughness = user_attributes.roughness;
    shading_info.normal = normal_world_space;
    shading_info.view_direction = view_direction;
    shading_info.f0 = lerp(float3(1.0, 1.0, 1.0) * 0.04, user_attributes.base_color.xyz, user_attributes.metallic);
    shading_info.nov = nov;
    shading_info.opacity = user_attributes.opacity;
    shading_info.shading_reflected = reflect(view_direction, normal_world_space);

    shading_info.clear_coat_info = fetch_clear_coat_info(
        nov,
        shading_info.shading_reflected,
        user_attributes.clear_coat,
        user_attributes.clear_coat_roughness);
    return shading_info;
}

VertexOutput vs_main_default(VertexIn vertex_in)
{
    float4x4 mvp = mul(global_constants.view_projection, constants.model);
    float4x4 bone_transform = make_identity_matrix();

#ifdef SKELETON_MAX_BONES
    int bone_index_0 = vertex_in.bone_ids[0];
    int bone_index_1 = vertex_in.bone_ids[1];
    int bone_index_2 = vertex_in.bone_ids[2];
    int bone_index_3 = vertex_in.bone_ids[3];
    bone_transform = skin_constants.bones[bone_index_0] * vertex_in.bone_weights[0];
    bone_transform += skin_constants.bones[bone_index_1] * vertex_in.bone_weights[1];
    bone_transform += skin_constants.bones[bone_index_2] * vertex_in.bone_weights[2];
    bone_transform += skin_constants.bones[bone_index_3] * vertex_in.bone_weights[3];
#endif

    float4x4 world_matrix = mul(constants.model, bone_transform);

    VertexOutput vertex_output;
#ifdef SKELETON_MAX_BONES
    vertex_output.position = mul(mvp, mul(bone_transform, float4(vertex_in.position, 1.0)));
    vertex_output.frag_position = mul(world_matrix, float4(vertex_in.position, 1.0)).xyz;
    vertex_output.normal = mul(transpose(inverse(world_matrix)), float4(vertex_in.normal, 0.0)).xyz;
#else
    vertex_output.position = mul(mvp, float4(vertex_in.position, 1.0));
    vertex_output.frag_position = mul(constants.model, float4(vertex_in.position, 1.0)).xyz;
    vertex_output.normal = vertex_in.normal;
#endif

    vertex_output.tex_coord0 = vertex_in.tex_coord0;
    vertex_output.vertex_color = vertex_in.vertex_color;

    vertex_output.tbn_t = mul(constants.model, float4(vertex_in.tangent, 0.0)).xyz;
    vertex_output.tbn_b = mul(constants.model, float4(vertex_in.bitangent, 0.0)).xyz;
    vertex_output.tbn_n = mul(constants.model, float4(vertex_output.normal, 0.0)).xyz;

    vertex_output.frag_position_at_light_space = mul(global_constants.light_space_matrix, float4(vertex_output.frag_position, 1.0));

    return vertex_output;
}

FragmentOutput fs_main_default(VertexOutput vertex_output, IFunc<UserAttributes> custom_get_user_attributes) 
{
    FragmentOutput fragment_output;

    UserAttributes user_attributes = custom_get_user_attributes();
    ShadingInfo shading_info = get_shading_info(user_attributes, vertex_output);

    float3 ibl_color = ibl_light(shading_info);

    float shadow = shadow_calculation(vertex_output.frag_position_at_light_space);

    float3 point_light_color = float3(0.0, 0.0, 0.0);
#ifdef SUPPORT_CLUSTER_LIGHTS
    int split_num = 10;
    int indirect_index = get_indirect_index_by_position(
        global_constants.camera_frustum_apply_transformation,
        vertex_output.frag_position, split_num, split_num, split_num);
    if (indirect_index > -1)
    {
        ClusterLightIndex cluster_light_index = cluster_light_indices[indirect_index];
        uint start = cluster_light_index.offset;
        uint end = start + cluster_light_index.count;
        for (uint i = start; i < end; i++)
        {
            uint light_index = cluster_lights[i];
            float3 light_color = point_light(point_lights[light_index], shading_info.normal, vertex_output.frag_position, shading_info.view_direction);
            light_color = max(float3(0.0), light_color);
            point_light_color = point_light_color + light_color;
        }
    }
#else
    for (uint i = 0; i < MAX_POINT_LIGHTS_NUM; i++)
    {
        if (i >= point_lights.available)
            break;
        point_light_color = point_light_color + point_light(point_lights.lights[i], shading_info.normal, vertex_output.frag_position, shading_info.view_direction);
    }
#endif

    float3 spot_light_color = float3(0.0, 0.0, 0.0);
    for (uint i = 0; i < MAX_SPOT_LIGHTS_NUM; i++)
    {
        if (i >= spot_lights.available)
            break;
        spot_light_color = spot_light_color + spot_light(spot_lights.lights[i], shading_info.normal, vertex_output.frag_position, shading_info.view_direction);
    }

    float3 light_color = ibl_color + point_light_color + spot_light_color;
    fragment_output.color = float4(lerp(light_color, light_color * float3(0.5), shadow), 1.0);
    fragment_output.color.w = user_attributes.opacity;

    if (global_constants.debug_shading == DEBUG_SHADING_TYPE_BASE_COLOR)
        fragment_output.color = float4(shading_info.base_color, 1.0);
    else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_METALLIC)
        fragment_output.color = float4(float3(shading_info.metallic), 1.0);
    else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_ROUGHNESS)
        fragment_output.color = float4(float3(shading_info.roughness), 1.0);
    else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_NORMAL)
        fragment_output.color = float4((shading_info.normal + 1.0) / 2.0, 1.0);
    else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_VERTEX_COLOR_0)
        fragment_output.color = vertex_output.vertex_color;
    else if (global_constants.debug_shading == DEBUG_SHADING_TYPE_SHADOW)
        fragment_output.color = float4(float3(lerp(1.0, 0.0, shadow)), 1.0);

    return fragment_output;
}
