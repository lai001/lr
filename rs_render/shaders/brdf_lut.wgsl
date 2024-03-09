#include "constants.wgsl"
#include "ibl_common.wgsl"

struct Constants
{
    sampleCount: u32,
};

struct CoordinateSystem
{
    x: vec3<f32>,
    y: vec3<f32>,
    z: vec3<f32>,
};

#ifndef TEXTURE_FORMAT
    #define TEXTURE_FORMAT rg16float
#endif

@group(0)
@binding(0)
var lut_texture: texture_storage_2d<TEXTURE_FORMAT, write>;

@group(1)
@binding(0)
var<uniform> constants: Constants;

fn convert_coordinate_system(v: vec3<f32>, coordinateSystem: CoordinateSystem) -> vec3<f32> {
    var C = mat4x4<f32>(vec4<f32>(coordinateSystem.x, 0.0),
                        vec4<f32>(coordinateSystem.y, 0.0),
                        vec4<f32>(coordinateSystem.z, 0.0),
                        vec4<f32>(0.0, 0.0, 0.0, 1.0));
    var v1 = C * vec4<f32>(v, 1.0);
    return v1.xyz;
}

fn integrate_brdf(in_n_dot_v: f32, roughness: f32) -> vec2<f32> {
    var n_dot_v = max(in_n_dot_v, 0.001);
    var v = vec3<f32>(sqrt(1.0 - n_dot_v * n_dot_v), 0.0, n_dot_v);

    var a: f32 = 0.0;
    var b: f32 = 0.0;

    var n = vec3<f32>(0.0, 0.0, 1.0);

    var tangent_vector = normalize(cross(vec3<f32>(1.0, 0.0, 0.0), n));
    var bitangent_vector = normalize(cross(n, tangent_vector));

    var sample_count = constants.sampleCount;
    for (var i: u32 = u32(0); i < sample_count ; i++) {
        var xi = hammersley_2d(i, sample_count);
        var ggx = importance_sample_ggx(xi, roughness);
        var coordinate_system: CoordinateSystem;
        coordinate_system.x = tangent_vector;
        coordinate_system.y = bitangent_vector;
        coordinate_system.z = n;
        var h = convert_coordinate_system(ggx, coordinate_system);

        var l = reflect(-v, h);

        var n_dot_l = max(0.0, l.z);
        var n_dot_h = max(0.0, h.z);
        var v_dot_h = max(0.0, dot(v, h));

        if (n_dot_l > 0.0) {
            var g = geometry_smith(n, v, l, roughness);
            var g_vis = (g * v_dot_h) / (n_dot_h * n_dot_v);
            var fc = pow((1.0 - v_dot_h), 5.0);
            a += (1.0 - fc) * g_vis;
            b += fc * g_vis;
        }
    }
    a /= f32(sample_count);
    b /= f32(sample_count);
    var result = vec2<f32>(a, b);
    return result;
}

@compute
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var lut_texture_dimensions = textureDimensions(lut_texture);
    var uv = (vec2<f32>(global_id.xy) + vec2(0.5)) / vec2<f32>(lut_texture_dimensions.xy);
    var v = integrate_brdf(uv.x, uv.y);
    var color = vec3<f32>(v.x, v.y, 0.0);
    textureStore(lut_texture, vec2<i32>(global_id.xy), vec4<f32>(color, 1.0));
}