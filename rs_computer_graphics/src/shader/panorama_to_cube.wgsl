const PI: f32 = 3.1415;
const TAU: f32 = 6.283;

@group(0)
@binding(0)
var equirectangular_texture: texture_2d<f32>;

@group(0)
@binding(1)
var cube_map: texture_storage_2d_array<rg11b10float, write>;

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
   var cube_map_texture_dimensions = textureDimensions(cube_map);
   var sample_uv = vec2<f32>(global_id.xy) / vec2<f32>(cube_map_texture_dimensions.xy) * 2.0 - 1.0;
   var sample_picker = get_sample_picker(global_id.z, sample_uv);
   var color = sample_equirectangular(equirectangular_texture, sample_picker, 0);
   textureStore(cube_map, vec2<i32>(global_id.xy), i32(global_id.z), color);
}