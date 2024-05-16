const PIXEL_OFFSET: f32 = 0.5;

struct Constants
{
    channel: i32,
    threshold: f32,
};

@group(0)
@binding(0)
var original_texture: texture_2d<f32>;

@group(0)
@binding(1)
var input_texture: texture_storage_2d<rgba16float, read>;

@group(0)
@binding(2)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0)
@binding(3)
var<uniform> constants: Constants;

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var input_texture_dimensions = textureDimensions(input_texture);
    var original_texture_color = textureLoad(original_texture, global_id.xy, 0);
    var input_texture_color = textureLoad(input_texture, global_id.xy);
    var threshold: f32;
    switch (constants.channel) {
        case 0: {
            threshold = original_texture_color.x;
        }
        case 1: {
            threshold = original_texture_color.y;
        }
        case 2: {
            threshold = original_texture_color.z;
        }
        case 3: {
            threshold = original_texture_color.w;
        }
        default {
            threshold = original_texture_color.x;
        }
    }

#ifdef USE_GRAYSCALE
    var distance: f32 = 0.0;
    if (threshold >= constants.threshold) {
        distance = -length(vec2<f32>(global_id.xy) + PIXEL_OFFSET - input_texture_color.xy * vec2<f32>(input_texture_dimensions.xy));
    	textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(distance, distance, distance, 1.0));
    } else {
        distance = length(vec2<f32>(global_id.xy) + PIXEL_OFFSET - input_texture_color.zw * vec2<f32>(input_texture_dimensions.xy));
    	textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(distance, distance, distance, 0.0));
    }
#else
    if (threshold >= constants.threshold) {
    	textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(input_texture_color.xy, 0.0, 1.0));
    } else {
    	textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(input_texture_color.zw, 0.0, 0.0));
    }
#endif
}