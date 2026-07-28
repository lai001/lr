#ifndef LIGHT_SLANG
#define LIGHT_SLANG

#ifndef MAX_POINT_LIGHTS_NUM
#define MAX_POINT_LIGHTS_NUM 2
#endif

#ifndef MAX_SPOT_LIGHTS_NUM
#define MAX_SPOT_LIGHTS_NUM 2
#endif

struct PointLight
{
    float3 position;
    float3 ambient;
    float3 diffuse;
    float3 specular;
    float constant;
    float linear;
    float quadratic;
};

struct PointLights
{
    PointLight lights[MAX_POINT_LIGHTS_NUM];
    uint available;
};

struct SpotLight
{
    PointLight light;
    float3 direction;
    float cut_off;
    float outer_cut_off;
};

struct SpotLights
{
    SpotLight lights[MAX_SPOT_LIGHTS_NUM];
    uint available;
};

float3 point_light(PointLight pt_light, float3 normal, float3 frag_position, float3 view_position)
{
    float3 light_dir = normalize(pt_light.position - frag_position);
    float diff = max(dot(normal, light_dir), 0.0);
    float3 diffuse = pt_light.diffuse;

    float3 view_dir = normalize(view_position - frag_position);
    float3 reflect_dir = reflect(-light_dir, normal);
    float spec = max(dot(view_dir, reflect_dir), 0.0);
    float3 specular = pt_light.specular * spec;

    float distance = length(pt_light.position - frag_position);
    float attenuation = 1.0 / (pt_light.constant + pt_light.linear * distance + pt_light.quadratic * (distance * distance));

    diffuse = diffuse * attenuation;
    specular = specular * attenuation;
    return diffuse + specular;
}

float3 spot_light(SpotLight spt_light, float3 normal, float3 frag_position, float3 view_position)
{
    float3 light_color = point_light(spt_light.light, normal, frag_position, view_position);
    float3 light_dir = normalize(spt_light.light.position - frag_position);
    float theta = dot(light_dir, normalize(-spt_light.direction));
    float epsilon = spt_light.cut_off - spt_light.outer_cut_off;
    float intensity = 1.0 - clamp((theta - spt_light.outer_cut_off) / epsilon, 0.0, 1.0);
    return light_color * intensity;
}

#endif
