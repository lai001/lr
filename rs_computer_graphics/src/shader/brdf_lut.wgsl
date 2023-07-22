const PI: f32 = 3.1415;
const TAU: f32 = 6.283;

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

@group(0)
@binding(0)
var lut_texture: texture_storage_2d<rgba32float, write>;

@group(1)
@binding(0)
var<uniform> constants: Constants;

// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
fn radical_inverse_vd_c(bits: u32) -> f32 {
	var bits = bits;
	bits = (bits << 16u) | (bits >> 16u);
	bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
	bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
	bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
	bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
	return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley_2d(i: u32, n: u32) -> vec2<f32> {
	return vec2<f32>(f32(i) / f32(n), radical_inverse_vd_c(i));
	// return float2((float)i / (float)n, van_der_corpus(i, 2u));
}

fn importance_sample_ggx(Xi: vec2<f32>, roughness: f32) -> vec3<f32> {
	var a = roughness * roughness;
	var phi = 2.0 * PI * Xi.x;
	var cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a * a - 1.0) * Xi.y));
	var sinTheta = sqrt(1.0 - cosTheta * cosTheta);
	var H: vec3<f32>;
	H.x = cos(phi) * sinTheta;
	H.y = sin(phi) * sinTheta;
	H.z = cosTheta;
	return H;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    var a = roughness;
    var k = (a * a) / 2.0;
    var nom = n_dot_v;
    var denom = n_dot_v * (1.0 - k) + k;
    return nom / denom;
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    var n_dot_v = max(0.0, dot(n, v));
    var n_dot_l = max(0.0, dot(n, l));
    var ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    var ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

fn convert_coordinate_system(v: vec3<f32>, coordinateSystem: CoordinateSystem) -> vec3<f32> {
	var C = mat4x4<f32>(vec4<f32>(coordinateSystem.x, 0.0), 
						vec4<f32>(coordinateSystem.y, 0.0), 
						vec4<f32>(coordinateSystem.z, 0.0), 
						vec4<f32>(0.0, 0.0, 0.0, 1.0));
	var v1 = C * vec4<f32>(v, 1.0);
	return v1.xyz;
}

fn integrate_brdf(n_dot_v: f32, roughness: f32) -> vec2<f32> {
    
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
    var uv = vec2<f32>(global_id.xy) / vec2<f32>(lut_texture_dimensions.xy);
    var v = integrate_brdf(uv.x, uv.y);
    var color = vec3<f32>(v.x, v.y, 0.0);
    textureStore(lut_texture, vec2<i32>(global_id.xy), vec4<f32>(color, 1.0));
}