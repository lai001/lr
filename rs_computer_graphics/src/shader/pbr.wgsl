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
    base_layer_ior: f32,
    clear_coat: f32,                
    clear_coat_roughness: f32,               
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

struct ClearCoatInfo {
	attenuation: f32,
	specular: vec3<f32>,  
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

fn cal_f0(ior: f32) -> f32 {
    return pow((ior - 1.5)  / (ior + 1.5), 2.0);
}

fn D_GGX(NoH: f32, a: f32) -> f32 {
    var a2 = a * a;
    var f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

fn V_Kelemen(LoH: f32) -> f32 {
    return 0.25 / (LoH * LoH);
}

fn F(H: vec3<f32>, V: vec3<f32>, F0: f32) -> f32 {
    var cosTheta = dot(H, V);
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

fn F3(H: vec3<f32>, V: vec3<f32>, F0: vec3<f32>) -> vec3<f32> {
    var cosTheta = dot(H, V);
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

fn F_Schlick(f0: f32, f90: f32, VoH: f32) -> f32 {
    var a = 1.0 - VoH;
    return f0 + (f90 - f0) * (a*a*a*a*a);
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

fn ibl_diffuse_color(shadingInfo: ShadingInfo, clear_coat_info: ClearCoatInfo, irradiance_texture: texture_cube<f32>) -> vec3<f32> {
    var irradiance = textureSample(irradiance_texture, base_color_sampler, shadingInfo.normal).xyz;
    var f = F3(shadingInfo.halfway_direction, shadingInfo.view_direction, shadingInfo.f0);
    var fac = mix(vec3<f32>(1.0) - f, vec3<f32>(0.0), shadingInfo.metalness);
    var diffuse_color = fac * shadingInfo.base_color.rgb * irradiance;
    return diffuse_color * clear_coat_info.attenuation;
}

fn ibl_specular_color(shadingInfo: ShadingInfo, clear_coat_info: ClearCoatInfo, light_reflection_vec: vec3<f32>, pre_filter_cube_map_texture: texture_cube<f32>, brdflut_texture: texture_2d<f32>) -> vec3<f32> {
    var levels = f32(textureNumLevels(pre_filter_cube_map_texture)) - 1.0;
    var lod = shadingInfo.roughness * levels;
    var pre_filter_value = textureSampleLevel(pre_filter_cube_map_texture, base_color_sampler, light_reflection_vec, lod).xyz;
    var brdf_value = textureSample(brdflut_texture, base_color_sampler, vec2<f32>(shadingInfo.nov, shadingInfo.roughness)).xy;
    var specular_color = (shadingInfo.f0 * brdf_value.x + brdf_value.y) * pre_filter_value;
    return specular_color * clear_coat_info.attenuation + clear_coat_info.specular;
}

fn IBL(shadingInfo: ShadingInfo, clear_coat_info: ClearCoatInfo, light_reflection_vec: vec3<f32>, irradiance_texture: texture_cube<f32>, pre_filter_cube_map_texture: texture_cube<f32>, brdflut_texture: texture_2d<f32>) -> vec3<f32> {
    var diffuse_color = ibl_diffuse_color(shadingInfo, clear_coat_info, irradiance_texture);
    var specular_color = ibl_specular_color(shadingInfo, clear_coat_info, light_reflection_vec, pre_filter_cube_map_texture, brdflut_texture);
    return diffuse_color + specular_color;
}

fn fetchClearCoatInfo(nov: f32, shading_reflected: vec3<f32>) -> ClearCoatInfo {
    var Fc = F_Schlick(0.04, 1.0, nov) * constants.clear_coat;
    var attenuation = 1.0 - Fc;
    var levels = f32(textureNumLevels(pre_filter_cube_map_texture)) - 1.0;
    var lod = levels * constants.clear_coat_roughness * (2.0 - constants.clear_coat_roughness);
    var pre_filter_value = textureSampleLevel(pre_filter_cube_map_texture, base_color_sampler, shading_reflected, lod).xyz;
    var specular = pre_filter_value * Fc;
    var info: ClearCoatInfo;
    info.attenuation = attenuation;
    info.specular = specular;
    return info;
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
    var nol = clamp(dot(normal_w, directional_light_direction), 0.0, 1.0);
    var d = D(normal_w, halfway_direction, roughness);
    var f = F3(halfway_direction, view_direction, mix(vec3<f32>(1.0, 1.0, 1.0) * 0.04, albedo_color.xyz, metalness));
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

    var shading_reflected = reflect(view_direction, normal_w);
    var clear_coat_info = fetchClearCoatInfo(nov, shading_reflected);

    var light_reflection_vec = reflect(directional_light_direction, shadingInfo.normal);
    var ibl_color = IBL(shadingInfo, clear_coat_info, shading_reflected, irradiance_texture, pre_filter_cube_map_texture, brdflut_texture);

    var color = directional_light_color + ibl_color;

    // color = (shadingInfo.normal + 1.0) / 2.0;

    return vec4<f32>(color, 1.0);
}