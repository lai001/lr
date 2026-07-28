#ifndef CLUSTER_LIGHT_SLANG
#define CLUSTER_LIGHT_SLANG

#include "camera_frustum.slang"

struct ClusterLightIndex
{
    uint offset;
    uint count;
};

float find_ratio(float3 start, float3 end, float3 p)
{
    float3 u = p - start;
    float3 v = end - start;
    float3 cast_vector = (dot(u, v) / pow(length(v), 2.0)) * v;
    return dot(normalize(cast_vector), normalize(v)) * length(cast_vector) / length(v);
}

int get_indirect_index_by_position(
    Frustum frustum,
    float3 position,
    int i_step,
    int j_step,
    int k_step)
{
    float3 start = (frustum.near_0 + frustum.near_1 + frustum.near_2 + frustum.near_3) / 4.0;
    float3 end = (frustum.far_0 + frustum.far_1 + frustum.far_2 + frustum.far_3) / 4.0;
    float ratio = find_ratio(start, end, position);
    if (ratio < 0.0 || ratio > 1.0)
        return int(-1);
    int index_k = int(floor(ratio * float(k_step)));

    float3 top_right = lerp(frustum.near_0, frustum.far_0, ratio);
    float3 bottom_right = lerp(frustum.near_1, frustum.far_1, ratio);
    float3 bottom_left = lerp(frustum.near_2, frustum.far_2, ratio);
    float3 top_left = lerp(frustum.near_3, frustum.far_3, ratio);

    int index_j = int(floor(find_ratio(top_right, bottom_right, position) * float(j_step)));
    int index_i = int(floor(find_ratio(top_left, top_right, position) * float(i_step)));

    return k_step * index_k + j_step * index_j + index_i;
}

#endif
