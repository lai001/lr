const PI: f32 = 3.1415;
const TAU: f32 = 6.283;

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
var prefilterMap: texture_storage_2d_array<rg11b10float, write>;

@group(1)
@binding(0)
var<uniform> pre_filter_environment_cube_map_constants: Constants;


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

fn van_der_corpus(n: u32, base: u32) -> f32 {
	var n = n;
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

fn get_sample_picker(face : u32, uv: vec2<f32>) -> vec3<f32> {
	var SamplePicker: vec3<f32>;
	switch(face)
	{
		case 0u:  {
			SamplePicker = vec3<f32>(1.0,  -uv.y, -uv.x);
		}
		case 1u: {
			SamplePicker = vec3<f32>(-1.0, -uv.y,  uv.x);
		}
		case 2u: {
			SamplePicker = vec3<f32>(uv.x, 1.0, uv.y);
		}
		case 3u: {
			SamplePicker = vec3<f32>(uv.x, -1.0, -uv.y);
		}
		case 4u: {
			SamplePicker = vec3<f32>(uv.x, -uv.y, 1.0);
		}
		case 5u: {
			SamplePicker = vec3<f32>(-uv.x, -uv.y, -1.0);
		}
		default {
			SamplePicker = vec3<f32>(1.0, 0.0, 0.0);
		}
	}
    return normalize(SamplePicker);
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

fn sample_from_3d_to_2d(sample_picker: vec3<f32>) -> vec2<f32> {
	var x = (atan2(sample_picker.z, sample_picker.x) + PI) / TAU;
	x = clamp(x, 0.0, 1.0);
    var y = clamp((acos(sample_picker.y) / PI), 0.0, 1.0);
    return vec2<f32>(x, y);
}

fn sample_equirectangular(texture: texture_2d<f32>, location: vec3<f32>, lod: i32) -> vec4<f32> {
	var texture_dimensions = textureDimensions(texture);
    let sample_picker = sample_from_3d_to_2d(location);
	var uv = vec2<i32>(i32(sample_picker.x * f32(texture_dimensions.x)), i32(sample_picker.y * f32(texture_dimensions.y)));
   	var color = textureLoad(texture, uv, lod);
    return color;
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
