const FXAA_REDUCE_MIN: f32 = 1.0 / 128.0;
const FXAA_REDUCE_MUL: f32 = 1.0 / 8.0;
const FXAA_SPAN_MAX: f32 = 8.0;

struct VertexIn {
    @builtin(vertex_index) ver_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord0: vec2<f32>,
    @location(1) @interpolate(flat) texture_size: vec2<f32>,
    @location(2) @interpolate(flat) inverse_texture_size: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var base_color_sampler: sampler;

@group(0) @binding(1) var input_texture: texture_2d<f32>;

fn rgb2luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
}

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
    var pos: array<vec2<f32>, 6>;
    var coords: array<vec2<f32>, 6>;
    pos[0] = vec2<f32>(-1.0, 1.0);
    coords[0] = vec2<f32>(0.0, 0.0);

    pos[1] = vec2<f32>(1.0, 1.0);
    coords[1] = vec2<f32>(1.0, 0.0);

    pos[2] = vec2<f32>(1.0, -1.0);
    coords[2] = vec2<f32>(1.0, 1.0);

    pos[3] = vec2<f32>(-1.0, 1.0);
    coords[3] = vec2<f32>(0.0, 0.0);

    pos[4] = vec2<f32>(-1.0, -1.0);
    coords[4] = vec2<f32>(0.0, 1.0);

    pos[5] = vec2<f32>(1.0, -1.0);
    coords[5] = vec2<f32>(1.0, 1.0);

    var vertex_output: VertexOutput;
    vertex_output.position = vec4<f32>(pos[vertex_in.ver_index], 0.0, 1.0);
    vertex_output.tex_coord0 = coords[vertex_in.ver_index];
    vertex_output.texture_size = vec2<f32>(textureDimensions(input_texture).xy);
    vertex_output.inverse_texture_size = 1.0 / vertex_output.texture_size;
    return vertex_output;
}

// https://github.com/BennyQBD/3DEngineCpp
fn applyFXAA(tex_coord0: vec2<f32>, tex: texture_2d<f32>, inverse_texture_size: vec2<f32>) -> vec4<f32> {
    var color: vec4<f32>;
    let tex_sampler = base_color_sampler;
	let tex_coord_offset: vec2<f32> = inverse_texture_size;
	let luma_tl: f32 = rgb2luma(textureSample(tex, tex_sampler, tex_coord0.xy + vec2<f32>(-1., -1.) * tex_coord_offset).xyz);
	let luma_tr: f32 = rgb2luma(textureSample(tex, tex_sampler, tex_coord0.xy + vec2<f32>(1., -1.) * tex_coord_offset).xyz);
	let luma_bl: f32 = rgb2luma(textureSample(tex, tex_sampler, tex_coord0.xy + vec2<f32>(-1., 1.) * tex_coord_offset).xyz);
	let luma_br: f32 = rgb2luma(textureSample(tex, tex_sampler, tex_coord0.xy + vec2<f32>(1., 1.) * tex_coord_offset).xyz);
	let luma_m: f32 = rgb2luma(textureSample(tex, tex_sampler, tex_coord0.xy).xyz);
	var dir: vec2<f32>;
	dir.x = -(luma_tl + luma_tr - (luma_bl + luma_br));
	dir.y = luma_tl + luma_bl - (luma_tr + luma_br);
	let dir_reduce: f32 = max((luma_tl + luma_tr + luma_bl + luma_br) * (FXAA_REDUCE_MUL * 0.25), FXAA_REDUCE_MIN);
	let inverse_dir_adjustment: f32 = 1. / (min(abs(dir.x), abs(dir.y)) + dir_reduce);
	dir = min(vec2<f32>(FXAA_SPAN_MAX, FXAA_SPAN_MAX), max(vec2<f32>(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX), dir * inverse_dir_adjustment)) * tex_coord_offset;
	let result1: vec3<f32> = 1. / 2. * (textureSample(tex, tex_sampler, tex_coord0.xy + dir * vec2<f32>(1. / 3. - 0.5)).xyz + textureSample(tex, tex_sampler, tex_coord0.xy + dir * vec2<f32>(2. / 3. - 0.5)).xyz);
	let result2: vec3<f32> = result1 * (1. / 2.) + 1. / 4. * (textureSample(tex, tex_sampler, tex_coord0.xy + dir * vec2<f32>(0. / 3. - 0.5)).xyz + textureSample(tex, tex_sampler, tex_coord0.xy + dir * vec2<f32>(3. / 3. - 0.5)).xyz);
	var luma_min: f32 = min(luma_m, min(min(luma_tl, luma_tr), min(luma_bl, luma_br)));
	var luma_max: f32 = max(luma_m, max(max(luma_tl, luma_tr), max(luma_bl, luma_br)));
	var luma_result2: f32 = rgb2luma(result2);
	if (luma_result2 < luma_min || luma_result2 > luma_max) {	
		color = vec4<f32>(result1, 1.);
	} else { 	
		color = vec4<f32>(result2, 1.);
	}
    return color;
} 

@fragment fn fs_main(vertex_output: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    fragment_output.color = applyFXAA(vertex_output.tex_coord0, input_texture, vertex_output.inverse_texture_size);
    return fragment_output;
}
