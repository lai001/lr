const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;

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
var cube_map: texture_storage_2d_array<rgba32float, write>;

@group(1)
@binding(0)
var<uniform> constants: Constants;

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

fn sample_from_3d_to_2d(sample_picker: vec3<f32>) -> vec2<f32> {
	var x = clamp((atan2(sample_picker.z, sample_picker.x) + PI) / TAU, 0.0, 1.0);
	var y = clamp(acos(sample_picker.y) / PI, 0.0, 1.0);
    return vec2<f32>(x, y);
}

fn sample_equirectangular(texture: texture_2d<f32>, location: vec3<f32>, lod: i32) -> vec4<f32> {
	var texture_dimensions = textureDimensions(texture, lod);
    var sample_picker = sample_from_3d_to_2d(location);
	sample_picker = sample_picker * vec2<f32>(texture_dimensions.xy);
	var uv = vec2<i32>(i32(sample_picker.x), i32(sample_picker.y));
   	var color = textureLoad(texture, uv, lod);
    return color;
}

fn hemisphere_sample_uniform(u: f32, v: f32) -> vec3<f32> {
    var phi = v * TAU;
    var cos_theta = 1.0 - u;
    var sin_theta = sqrt((1.0 - cos_theta * cos_theta));
    return vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
}

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
	var sample_uv = vec2<f32>(global_id.xy) / vec2<f32>(cube_map_texture_dimensions.xy) * 2.0 - 1.0;
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