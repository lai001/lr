const PI: f32 = 3.1415;
const TAU: f32 = 6.283;

struct Constants {                              
    directional_light: DirectionalLight,        
    point_light: PointLight,                    
    spot_light: SpotLight,                      
    model: mat4x4<f32>,                         
    view: mat4x4<f32>,                          
    projection: mat4x4<f32>,                    
    view_position: vec3<f32>,                   
    roughness_factor: f32,                      
    metalness_factor: f32,                      
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

struct DirectionalLight {                      
	direction: vec3<f32>,                      
	ambient: vec3<f32>,                        
	diffuse: vec3<f32>,                        
	specular: vec3<f32>,                       
};

struct PointLight {                            
	position: vec3<f32>,                       
	ambient: vec3<f32>,                        
	diffuse: vec3<f32>,                        
	specular: vec3<f32>,                       
	constant: f32,                             
	linear: f32,                               
	quadratic: f32,                            
};

struct SpotLight {                             
	position: vec3<f32>,                       
	direction: vec3<f32>,                      
	ambient: vec3<f32>,                        
	diffuse: vec3<f32>,                        
	specular: vec3<f32>,                       
	cut_off: f32,                              
	outer_cut_off: f32,                        
	constant: f32,                             
	linear: f32,                               
	quadratic: f32,                            
};

struct ShadingInfo {
	normal: vec3<f32>,
	view_direction: vec3<f32>,
	halfway_direction: vec3<f32>,
	base_color: vec4<f32>,
	metalness: f32,
	roughness: f32,
	nov: f32,
	noh: f32,
	f0: vec3<f32>,
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

@group(2) @binding(1) 
var base_color_sampler_non_filtering : sampler;

fn D(N: vec3<f32>, H: vec3<f32>, a: f32) -> f32 {
    var a2 = a * a;
    var NdotH = max(dot(N, H), 0.0);
    var NdotH2 = NdotH * NdotH;

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
  
fn G(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, k: f32) -> f32 {
    var NdotV = max(dot(N, V), 0.0);
    var NdotL = max(dot(N, L), 0.0);
    var ggx1 = SubG(NdotV, k);
    var ggx2 = SubG(NdotL, k);
    return ggx1 * ggx2;
}

fn IBL(shadingInfo: ShadingInfo, light_reflection_vec: vec3<f32>, irradiance_texture: texture_cube<f32>, pre_filter_cube_map_texture: texture_cube<f32>, brdflut_texture: texture_2d<f32>) -> vec3<f32> {
    var irradiance = textureSample(irradiance_texture, base_color_sampler_non_filtering, shadingInfo.normal).xyz;
    var f = F(shadingInfo.halfway_direction, shadingInfo.view_direction, shadingInfo.f0);
    var fac = mix(vec3<f32>(1.0) - f, vec3<f32>(0.0), shadingInfo.metalness);
    var diffuse_color = fac * shadingInfo.base_color.rgb * irradiance;
    var levels = textureNumLevels(pre_filter_cube_map_texture);
    var pre_filter_value = textureSampleLevel(pre_filter_cube_map_texture, base_color_sampler_non_filtering, light_reflection_vec, shadingInfo.roughness * f32(levels)).xyz;
    var brdf_value = textureSample(brdflut_texture, base_color_sampler_non_filtering, vec2<f32>(shadingInfo.nov, shadingInfo.roughness)).xy;
    var specular_color = (shadingInfo.f0 * brdf_value.x + brdf_value.y) * pre_filter_value;
    return diffuse_color + specular_color;
}

fn GetNormal(normal_texture: texture_2d<f32>, tex_coord: vec2<f32>, tbn: mat3x3<f32>) -> vec3<f32> {
    var normal = normalize(textureSample(normal_texture, base_color_sampler, tex_coord).xyz * 2.0 - 1.0);
    var normal_w = normalize(tbn * normal);
    return normal_w;
}

@vertex 
fn vs_main(
    @location(0) vertex_color: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) tex_coord: vec2<f32>,
) -> VertexOutput {
    let mv = constants.view * constants.model;
    let mvp = constants.projection * mv;
    var vertex_output: VertexOutput;
    vertex_output.position = mvp * vec4<f32>(position, 1.0);
    vertex_output.tex_coord = tex_coord;
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

    var albedo_color = textureSample(albedo_texture, base_color_sampler, vertex_output.tex_coord);
    var metalness = textureSample(metallic_texture, base_color_sampler, vertex_output.tex_coord).x * constants.metalness_factor;
    var roughness = textureSample(roughness_texture, base_color_sampler, vertex_output.tex_coord).x * constants.roughness_factor;
    var normal_w = GetNormal(normal_texture, vertex_output.tex_coord, tbn);
    var directional_light_direction = normalize(-constants.directional_light.direction);
    var view_direction = normalize(constants.view_position - vertex_output.frag_position.xyz);
    var halfway_direction = normalize(directional_light_direction + view_direction);
    var nov = dot(normal_w, view_direction);
    var noh = dot(normal_w, halfway_direction);
    var nol = dot(normal_w, directional_light_direction);
    var d = D(normal_w, halfway_direction, roughness);
    var f = F(halfway_direction, view_direction, mix(vec3<f32>(1.0, 1.0, 1.0) * 0.04, albedo_color.xyz, metalness));
    var g = G(normal_w, view_direction, directional_light_direction, pow((roughness + 1.0), 2.0) / 8.0);
    var sbrdf = d * f * g / (4.0 * max(nov * nol, 0.01));
    var dbrdf = mix(vec3<f32>(1.0, 1.0, 1.0) - f, vec3<f32>(0.0, 0.0, 0.0), metalness) * albedo_color.xyz;
    var directional_light_color = (dbrdf + sbrdf) * nol;

    var shadingInfo: ShadingInfo;
    shadingInfo.base_color = albedo_color;
    shadingInfo.metalness = metalness;
    shadingInfo.roughness = roughness;
    shadingInfo.normal = normal_w;
    shadingInfo.view_direction = view_direction;
    shadingInfo.halfway_direction = halfway_direction;
    shadingInfo.f0 = mix(vec3<f32>(1.0, 1.0, 1.0) * 0.04, albedo_color.xyz, metalness);
    shadingInfo.nov = nov;
    shadingInfo.noh = noh;

    var light_reflection_vec = reflect(directional_light_direction, shadingInfo.normal);
    var ibl_color = IBL(shadingInfo, light_reflection_vec, irradiance_texture, pre_filter_cube_map_texture, brdflut_texture);

    var color = directional_light_color + ibl_color;

    // color = (shadingInfo.normal + 1.0) / 2.0;

    return vec4<f32>(color, 1.0);
}