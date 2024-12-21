#ifndef CLUSTER_LIGHT_WGSL
#define CLUSTER_LIGHT_WGSL

#include "camera_frustum.wgsl"

struct ClusterLightIndex {
    offset: u32,
    count: u32,
}

fn find_ratio(start: vec3<f32>, end: vec3<f32>, p: vec3<f32>) -> f32 {
    var u = p - start;
    var v = end - start;
    var cast_vector = (dot(u, v) / pow(length(v), 2.0)) * v;
    return dot(normalize(cast_vector), normalize(v)) * length(cast_vector) / length(v);
}

fn get_indirect_index_by_position(
    frustum: Frustum, 
    position: vec3<f32>,
    i_step: i32,
    j_step: i32,
    k_step: i32
) -> i32 {
    var start = (frustum.near_0 + frustum.near_1 + frustum.near_2 + frustum.near_3) / 4.0;
    var end = (frustum.far_0 + frustum.far_1 + frustum.far_2 + frustum.far_3) / 4.0;
    var ratio = find_ratio(start, end, position);
    if (ratio < 0.0 || ratio > 1.0) {
        return i32(-1);
    }
    var index_k = i32(floor(ratio * f32(k_step)));

    var top_right = mix(frustum.near_0, frustum.far_0, ratio);
    var bottom_right = mix(frustum.near_1, frustum.far_1, ratio);
    var bottom_left = mix(frustum.near_2, frustum.far_2, ratio);
    var top_left = mix(frustum.near_3, frustum.far_3, ratio);
    
    var index_j = i32(floor(find_ratio(top_right, bottom_right, position) * f32(j_step)));
    var index_i = i32(floor(find_ratio(top_left, top_right, position) * f32(i_step)));

    return k_step * index_k + j_step * index_j + index_i;
}

#endif