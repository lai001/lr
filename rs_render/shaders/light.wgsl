#ifndef LIGHT_WGSL
#define LIGHT_WGSL

struct PointLight {
    position: vec3<f32>,

    ambient: vec3<f32>,
    diffuse: vec3<f32>,
    specular: vec3<f32>,

    constant: f32,
    linear: f32,
    quadratic: f32,
}

struct PointLights {
    lights: array<PointLight, MAX_POINT_LIGHTS_NUM>,
    available: u32,
}

struct SpotLight {
    light: PointLight,
    direction: vec3<f32>,
    cut_off: f32,
    outer_cut_off: f32,
};

struct SpotLights {
    lights: array<SpotLight, MAX_SPOT_LIGHTS_NUM>,
    available: u32,
}

fn point_light(point_light: PointLight, normal: vec3<f32>, frag_position: vec3<f32>, view_position: vec3<f32>) -> vec3<f32> {
    var light_dir: vec3<f32> = normalize(point_light.position - frag_position);
    var diff: f32 = max(dot(normal, light_dir), 0.0);
    var diffuse: vec3<f32> = point_light.diffuse;

    var view_dir: vec3<f32> = normalize(view_position - frag_position);
    var reflect_dir: vec3<f32> = reflect(-light_dir, normal);
    var spec: f32 = max(dot(view_dir, reflect_dir), 0.0);
    var specular: vec3<f32> = point_light.specular * spec;

    var distance: f32 = length(point_light.position - frag_position);
    var attenuation: f32 = 1.0 / (point_light.constant + point_light.linear * distance + point_light.quadratic * (distance * distance));

    diffuse = diffuse * attenuation;
    specular = specular * attenuation;
    return diffuse + specular;
}

fn spot_light(spot_light: SpotLight, normal: vec3<f32>, frag_position: vec3<f32>, view_position: vec3<f32>) -> vec3<f32> {
    var light_color = point_light(spot_light.light, normal, frag_position, view_position);
    var light_dir: vec3<f32> = normalize(spot_light.light.position - frag_position);
    var theta: f32 = dot(light_dir, normalize(-spot_light.direction));
    var epsilon: f32 = spot_light.cut_off - spot_light.outer_cut_off;
    var intensity: f32 = 1.0 - clamp((theta - spot_light.outer_cut_off) / epsilon, 0.0, 1.0);
    return light_color * intensity;
}

#endif