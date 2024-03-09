#include "constants.wgsl"
#include "ibl_common.wgsl"
#include "sample_equirectangular.wgsl"

#ifndef TEXTURE_FORMAT
    #define TEXTURE_FORMAT rg11b10float
#endif

struct CoordinateSystem
{
    x: vec3<f32>,
    y: vec3<f32>,
    z: vec3<f32>,
};

struct Constants
{
    sampleCount: u32,
};

@group(0)
@binding(0)
var equirectangular_texture: texture_2d<f32>;

@group(0)
@binding(1)
var cube_map: texture_storage_2d_array<TEXTURE_FORMAT, write>;

@group(1)
@binding(0)
var<uniform> constants: Constants;

fn hemisphere_sample_uniform(u: f32, v: f32) -> vec3<f32> {
    var phi = v * TAU;
    var cos_theta = 1.0 - u;
    var sin_theta = sqrt((1.0 - cos_theta * cos_theta));
    return vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
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
    var cube_map_texture_dimensions = textureDimensions(cube_map);
    var sample_uv = (vec2<f32>(global_id.xy) + vec2(0.5)) / vec2<f32>(cube_map_texture_dimensions.xy) * 2.0 - 1.0;
    var sample_picker = get_sample_picker(global_id.z, sample_uv);
    var up_vector = vec3<f32>(0.0, 1.0, 0.0);
    var tangent_vector = normalize(cross(sample_picker, up_vector));
    var bitangent_vector = normalize(cross(sample_picker, tangent_vector));
    var irradiance = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = u32(0); i < constants.sampleCount; i++) {
        var h = hammersley_2d(i, constants.sampleCount);
        var r = hemisphere_sample_uniform(h.x, h.y);
        var coodrdinate:CoordinateSystem;
        coodrdinate.x = bitangent_vector;
        coodrdinate.y = tangent_vector;
        coodrdinate.z = sample_picker;
        var l = convert_coordinate_system(r, coodrdinate);
        var source_pixel = sample_equirectangular(equirectangular_texture, l, 0).xyz;
        var add = 2.0 * source_pixel * max(0.0, dot(l, sample_picker));
        irradiance = irradiance + add;
    }
    irradiance = irradiance / f32(constants.sampleCount);
    irradiance = clamp(irradiance, vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(cube_map, vec2<i32>(global_id.xy), i32(global_id.z), vec4<f32>(irradiance, 1.0));
}