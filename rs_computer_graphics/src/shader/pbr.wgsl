const PI: f32 = 3.1415;
const TAU: f32 = 6.283;

struct Constants {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    roughness: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
	@location(0) frag_position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) tex_coord: vec2<f32>,
	@location(3) vertex_color: vec4<f32>,
	@location(4) tbn_t: vec3<f32>,
	@location(5) tbn_b: vec3<f32>,
	@location(6) tbn_n: vec3<f32>,
};

@group(0) @binding(0) 
var<uniform> constants: Constants;

@group(1) @binding(0) 
var albedo_texture: texture_2d<f32>;

@group(1) @binding(1) 
var normal_texture: texture_2d<f32>;

@group(1) @binding(2) 
var metallic_texture: texture_2d<f32>;

@group(1) @binding(3) 
var roughness_texture: texture_2d<f32>;

@group(1) @binding(4) 
var brdflut_texture: texture_2d<f32>;

@group(1) @binding(5) 
var pre_filter_cube_map_texture: texture_cube<f32>;

@group(1) @binding(6) 
var irradiance_texture: texture_cube<f32>;

@group(2) @binding(0) 
var base_color_sampler : sampler;

fn D(N: vec3<f32>, H: vec3<f32>, a: f32) -> f32 {
    var a2 = a*a;
    var NdotH = max(dot(N, H), 0.0);
    var NdotH2 = NdotH*NdotH;
	
    var nom = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return nom / denom;
}

fn F(H: vec3<f32>, V: vec3<f32>, F0: vec3<f32>) -> vec3<f32> {
	var cosTheta = dot(H, V);
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

fn SubG(InAngle: f32, k: f32) -> f32 {
    var nom = InAngle;
    var denom = InAngle * (1.0 - k) + k;
    return nom / denom;
}
  
fn G(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, k: f32) -> f32
{
    var NdotV = max(dot(N, V), 0.0);
    var NdotL = max(dot(N, L), 0.0);
    var ggx1 = SubG(NdotV, k); 
    var ggx2 = SubG(NdotL, k); 
    return ggx1 * ggx2;
}

@vertex 
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) tex_coord: vec2<f32>,
    @location(5) vertex_color: vec4<f32>,
) -> VertexOutput {
    let mv = constants.view * constants.model;
    let mvp = constants.projection * mv;
    var vertex_output: VertexOutput;
    vertex_output.tex_coord = tex_coord;
    vertex_output.position = mvp * vec4<f32>(position, 1.0);
    vertex_output.vertex_color = vertex_color;
    vertex_output.normal = normal;
    vertex_output.frag_position = (constants.model * vec4<f32>(position, 1.0)).xyz;
    vertex_output.tbn_t = (constants.model * vec4<f32>(tangent, 0.0)).xyz;
    vertex_output.tbn_b = (constants.model * vec4<f32>(bitangent, 0.0)).xyz;
    vertex_output.tbn_n = (constants.model * vec4<f32>(normal, 0.0)).xyz;
    return vertex_output;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    var tbn = mat3x3<f32>(
        vertex_output.tbn_t,
        vertex_output.tbn_b,
        vertex_output.tbn_n,
    );

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}