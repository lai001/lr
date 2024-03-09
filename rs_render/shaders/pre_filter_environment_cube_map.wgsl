#include "constants.wgsl"
#include "ibl_common.wgsl"
#include "sample_equirectangular.wgsl"

#ifndef TEXTURE_FORMAT
    #define TEXTURE_FORMAT rg11b10float
#endif

struct Constants
{
    roughness: f32,
    sampleCount: u32,
};

struct CoordinateSystem
{
    x: vec3<f32>,
    y: vec3<f32>,
    z: vec3<f32>,
};

@group(0)
@binding(0)
var equirectangular_texture: texture_2d<f32>;

@group(0)
@binding(1)
var prefilterMap: texture_storage_2d_array<TEXTURE_FORMAT, write>;

@group(1)
@binding(0)
var<uniform> pre_filter_environment_cube_map_constants: Constants;

fn van_der_corpus(in_n: u32, base: u32) -> f32 {
    var n = in_n;
    var invBase = 1.0 / f32(base);
    var denom: f32  = 1.0;
    var result  = 0.0;
    for (var i: u32 = u32(0); i < u32(32); i++) {
        if(n > u32(0)) {
            denom   = f32(n) % f32(2.0);
            result += denom * invBase;
            invBase = invBase / 2.0;
            n       = u32(f32(n) / 2.0);
        }
    }
    return result;
}

fn make_coordinate_system(N: vec3<f32>) -> CoordinateSystem {
    var UpVector = vec3<f32>(0.0, 1.0, 0.0);
    var system: CoordinateSystem;
    system.x = normalize(cross(N, UpVector));
    system.y = normalize(cross(N, system.x));
    system.z = N;
    return system;
}

fn convert_coordinate_system(v: vec3<f32>, coordinateSystem: CoordinateSystem) -> vec3<f32> {
    var C = mat4x4<f32>(vec4<f32>(coordinateSystem.x, 0.0),
                        vec4<f32>(coordinateSystem.y, 0.0),
                        vec4<f32>(coordinateSystem.z, 0.0),
                        vec4<f32>(0.0, 0.0, 0.0, 1.0));
    var v1 = C * vec4<f32>(v, 1.0);
    return v1.xyz;
}

@compute
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var cube_map_texture_dimensions = textureDimensions(prefilterMap);
    var sample_uv = (vec2<f32>(global_id.xy) + vec2(0.5)) / vec2<f32>(cube_map_texture_dimensions.xy) * 2.0 - 1.0;

    var N = get_sample_picker(global_id.z, sample_uv);

    var V = N;
    var coordinateSystem = make_coordinate_system(N);
    var prefilteredColor: vec3<f32> = vec3<f32>(0.0);
    var totalWeight: f32 = 0.0;
    var sampleCount = pre_filter_environment_cube_map_constants.sampleCount;//u32(1024);
    for (var i: u32 = u32(0); i < sampleCount ; i++) {
        var Xi = hammersley_2d(i, sampleCount);
        var ggx = importance_sample_ggx(Xi, pre_filter_environment_cube_map_constants.roughness);
        var H = convert_coordinate_system(ggx, coordinateSystem);
        var L = reflect(-H, V);
        var NdotL = dot(N, L);
        if (NdotL > 0.0) {
            prefilteredColor += sample_equirectangular(equirectangular_texture, L, 0).xyz * NdotL;
            totalWeight += NdotL;
        }
    }
    prefilteredColor /= totalWeight;
    textureStore(prefilterMap, vec2<i32>(global_id.xy), i32(global_id.z), vec4<f32>(prefilteredColor, 1.0));
}
