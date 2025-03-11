#include "global_constants.wgsl"

struct AABB {
    min: vec3<f32>,
    max: vec3<f32>,
    transformation: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;

@group(0) @binding(1) var depth_texture_depth_2d: texture_depth_2d;

@group(0) @binding(2) var<storage, read> boxes: array<AABB>;

@group(0) @binding(3) var<storage, read_write> results: array<u32>;

fn is_not_pass(v0: f32, v1: f32) -> bool {
    if (v0 > v1) {
        return true;
    } else {
        return false;
    }
}

fn is_valid_f(x: f32) -> bool {
    if (x >= -1.0 && x <= 1.0) {
        return true;
    } else {
        return false;
    }
}

fn is_valid(p: vec3<f32>) -> bool {
    return is_valid_f(p.x) && is_valid_f(p.y) && is_valid_f(p.z);
}

fn is_not_valid(p: vec3<f32>) -> bool {
    return !is_valid(p);
}

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var length_of_boxes = arrayLength(&boxes);
    var length_of_results = arrayLength(&results);
    if (length_of_boxes != length_of_results) {
        return;
    }
    var depth_texture_depth_2d_dimensions = textureDimensions(depth_texture_depth_2d);
    var width = depth_texture_depth_2d_dimensions.x;
    var height = depth_texture_depth_2d_dimensions.y;

    var i = global_id.x;
    var box = boxes[i];
    var mvp = global_constants.view_projection * box.transformation;
    var v0 = mvp * vec4<f32>(box.min.x, box.min.y, box.min.z, 1.0);
    var v1 = mvp * vec4<f32>(box.max.x, box.min.y, box.min.z, 1.0);
    var v2 = mvp * vec4<f32>(box.max.x, box.max.y, box.min.z, 1.0);
    var v3 = mvp * vec4<f32>(box.min.x, box.max.y, box.min.z, 1.0);
    var v4 = mvp * vec4<f32>(box.min.x, box.min.y, box.max.z, 1.0);
    var v5 = mvp * vec4<f32>(box.max.x, box.min.y, box.max.z, 1.0);
    var v6 = mvp * vec4<f32>(box.max.x, box.max.y, box.max.z, 1.0);
    var v7 = mvp * vec4<f32>(box.min.x, box.max.y, box.max.z, 1.0);

    v0 = v0 / v0.w;
    v1 = v1 / v1.w;
    v2 = v2 / v2.w;
    v3 = v3 / v3.w;
    v4 = v4 / v4.w;
    v5 = v5 / v5.w;
    v6 = v6 / v6.w;
    v7 = v7 / v7.w;
    results[i] = u32(1);
    if (is_not_valid(v0.xyz) ||
        is_not_valid(v1.xyz) ||
        is_not_valid(v2.xyz) ||
        is_not_valid(v3.xyz) ||
        is_not_valid(v4.xyz) ||
        is_not_valid(v5.xyz) ||
        is_not_valid(v6.xyz) ||
        is_not_valid(v7.xyz)) {
        return;
    }

    var uv0 = vec2<i32>(((v0.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv1 = vec2<i32>(((v1.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv2 = vec2<i32>(((v2.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv3 = vec2<i32>(((v3.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv4 = vec2<i32>(((v4.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv5 = vec2<i32>(((v5.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv6 = vec2<i32>(((v6.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));
    var uv7 = vec2<i32>(((v7.xy + 1.0) * 0.5) * vec2<f32>(f32(width), f32(height)));

    var depth0 = textureLoad(depth_texture_depth_2d, uv0, 0);
    var depth1 = textureLoad(depth_texture_depth_2d, uv1, 0);
    var depth2 = textureLoad(depth_texture_depth_2d, uv2, 0);
    var depth3 = textureLoad(depth_texture_depth_2d, uv3, 0);
    var depth4 = textureLoad(depth_texture_depth_2d, uv4, 0);
    var depth5 = textureLoad(depth_texture_depth_2d, uv5, 0);
    var depth6 = textureLoad(depth_texture_depth_2d, uv6, 0);
    var depth7 = textureLoad(depth_texture_depth_2d, uv7, 0);

    var is_not_pass = is_not_pass(v0.z, depth0) &&
                        is_not_pass(v1.z, depth1) &&
                        is_not_pass(v2.z, depth2) &&
                        is_not_pass(v3.z, depth3) &&
                        is_not_pass(v4.z, depth4) &&
                        is_not_pass(v5.z, depth5) &&
                        is_not_pass(v6.z, depth6) &&
                        is_not_pass(v7.z, depth7);
    if (is_not_pass) {
        results[i] = u32(0);
    }
}