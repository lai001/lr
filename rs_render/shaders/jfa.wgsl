const PIXEL_OFFSET: f32 = 0.5;

const ARRAY_SIZE: i32 = 8;
const DIRECTIONS: array<vec2<i32>, ARRAY_SIZE> = array(
    vec2<i32>(-1, -1),
    vec2<i32>(-1, 0),
    vec2<i32>(-1, 1),
    vec2<i32>(0, -1),
    vec2<i32>(0, 1),
    vec2<i32>(1, -1),
    vec2<i32>(1, 0),
    vec2<i32>(1, 1),
);

struct Constants
{
    step: vec2<f32>,
};

@group(0)
@binding(0)
var input_texture: texture_storage_2d<rgba16float, read>;

@group(0)
@binding(1)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0)
@binding(2)
var<uniform> constants: Constants;

fn jfa_outside(input_tex: vec4<f32>, idxy: vec2<f32>) -> vec4<f32> {
	var directions = DIRECTIONS;

	var outputTex = input_tex;
    var input_texture_dimensions = textureDimensions(input_texture);

	if (input_tex.x != -1) {
		var nearest_uv: vec2<f32> = input_tex.zw;
		var min_distance: f32 = 1e16;

		if (input_tex.z != -1.0)
		{
			min_distance = length(idxy + PIXEL_OFFSET - nearest_uv * vec2<f32>(input_texture_dimensions.xy));
		}

		var has_min: bool = false;
		for (var i = i32(0); i < ARRAY_SIZE; i++) {
			let direction = directions[i];
			var sample_offset: vec2<f32> = idxy + vec2<f32>(direction) * constants.step;
			sample_offset = clamp(sample_offset, vec2<f32>(0.0), vec2<f32>(input_texture_dimensions.xy) - vec2<f32>(1.0));
			var offset_texture: vec4<f32> = textureLoad(input_texture, vec2<i32>(sample_offset));

			if (offset_texture.z != -1.0)
			{
				var temp_uv: vec2<f32> = offset_texture.zw;
				var temp_distance: f32 = length(idxy + PIXEL_OFFSET - temp_uv * vec2<f32>(input_texture_dimensions.xy));
				if (temp_distance < min_distance)
				{
					has_min = true;
					min_distance = temp_distance;
					nearest_uv = temp_uv;
				}
			}
		}

		if (has_min) {
			outputTex = vec4<f32>(input_tex.xy, nearest_uv);
		}
	}
	return outputTex;
}

fn jfa_inside(input_tex: vec4<f32>, idxy: vec2<f32>) -> vec4<f32> {
	var directions = DIRECTIONS;

	var outputTex: vec4<f32> = input_tex;
    var input_texture_dimensions = textureDimensions(input_texture);

	if (input_tex.z != -1.0) {
		var nearest_uv: vec2<f32> = input_tex.xy;
		var min_distance: f32 = 1e16;

		if (input_tex.x != -1.0) {
			min_distance = length(idxy + PIXEL_OFFSET - nearest_uv * vec2<f32>(input_texture_dimensions.xy));
		}

		var has_min = false;
		for (var i = i32(0); i < ARRAY_SIZE; i++) {
			let direction = directions[i];
			var sample_offset: vec2<f32>  = idxy + vec2<f32>(direction) * constants.step;
			sample_offset = clamp(sample_offset, vec2<f32>(0.0), vec2<f32>(input_texture_dimensions.xy) - 1.0);
			var offset_texture:  vec4<f32> = textureLoad(input_texture, vec2<i32>(sample_offset));

			if (offset_texture.x != -1.0) {
				var temp_uv: vec2<f32> = offset_texture.xy;
				var temp_distance: f32 = length(idxy + PIXEL_OFFSET - temp_uv * vec2<f32>(input_texture_dimensions.xy));
				if (temp_distance < min_distance) {
					has_min = true;
					min_distance = temp_distance;
					nearest_uv = temp_uv;
				}
			}
		}

		if (has_min) {
			outputTex = vec4<f32>(nearest_uv, input_tex.zw);
		}
	}
	return outputTex;
}

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var input_texture_dimensions = textureDimensions(input_texture);
    var sample_position = vec2<f32>(vec2<f32>(global_id.xy) + PIXEL_OFFSET);
    if (sample_position.x >= f32(input_texture_dimensions.x)) {
        return;
    }
    if (sample_position.y >= f32(input_texture_dimensions.y)) {
        return;
    }
    var input_texture_color = textureLoad(input_texture, global_id.xy);
	var out_side: vec4<f32> = jfa_outside(input_texture_color, vec2<f32>(global_id.xy));
	textureStore(output_texture, vec2<i32>(global_id.xy), jfa_inside(out_side, vec2<f32>(global_id.xy)));
}