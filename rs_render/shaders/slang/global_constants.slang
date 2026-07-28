#ifndef GLOBAL_CONSTANTS_SLANG
#define GLOBAL_CONSTANTS_SLANG

#include "camera_frustum.slang"

static const int DEBUG_SHADING_TYPE_NONE = 0;
static const int DEBUG_SHADING_TYPE_BASE_COLOR = 1;
static const int DEBUG_SHADING_TYPE_METALLIC = 2;
static const int DEBUG_SHADING_TYPE_ROUGHNESS = 3;
static const int DEBUG_SHADING_TYPE_NORMAL = 4;
static const int DEBUG_SHADING_TYPE_VERTEX_COLOR_0 = 5;
static const int DEBUG_SHADING_TYPE_SHADOW = 6;

struct GlobalConstants
{
    float4x4 view;
    float4x4 projection;
    float4x4 view_projection;
    float4x4 light_space_matrix;
    Frustum camera_frustum_apply_transformation;
    float3 view_position;
    float physical_texture_size;
    float tile_size;
    int is_enable_virtual_texture;
    float scene_factor;
    float feedback_bias;
    int debug_shading;
    float time;
};

#endif
