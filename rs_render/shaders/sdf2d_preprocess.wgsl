
const PIXEL_OFFSET: f32 = 0.5;

struct Constants
{
    channel: i32,
    threshold: f32,
};

@group(0)
@binding(0)
var input_texture: texture_2d<f32>;

@group(0)
@binding(1)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0)
@binding(2)
var<uniform> constants: Constants;

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var input_texture_dimensions = textureDimensions(input_texture);
    var texture_size = vec2<f32>(f32(input_texture_dimensions.x), f32(input_texture_dimensions.y));
    var inverse_texture_size = 1.0 / texture_size;
    var sample_position = vec2<f32>(vec2<f32>(global_id.xy) + PIXEL_OFFSET);
    if (sample_position.x >= f32(input_texture_dimensions.x)) {
        return;
    }
    if (sample_position.y >= f32(input_texture_dimensions.y)) {
        return;
    }
    var input_texture_color = textureLoad(input_texture, global_id.xy, 0);

    var threshold: f32;
    switch (constants.channel) {
        case 0: {
            threshold = input_texture_color.x;
        }
        case 1: {
            threshold = input_texture_color.y;
        }
        case 2: {
            threshold = input_texture_color.z;
        }
        case 3: {
            threshold = input_texture_color.w;
        }
        default {
            threshold = input_texture_color.x;
        }
    }
    if (threshold >= constants.threshold) {
        textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(-1.0, -1.0, sample_position.x * inverse_texture_size.x, sample_position.y * inverse_texture_size.y));
    } else {
        textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(sample_position.x * inverse_texture_size.x, sample_position.y * inverse_texture_size.y, -1.0, -1.0));
    }
}