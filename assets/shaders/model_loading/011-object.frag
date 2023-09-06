#version 330 core

out vec4 frag_color;

struct Material {
    sampler2D diffuse_map;
    sampler2D specular_map;
    sampler2D normal_map;
    // float shininess;
};

struct PointLight {
    vec3 position;
    vec3 color;

    float attenuation_linear;
    float attenuation_quadratic;
};  
#define POINT_LIGHTS_NUM 4

struct SpotLight {
    vec3 color;

    vec3 position;
    vec3 direction;
    float cutoff;
    float outer_cutoff;
  
    float attenuation_linear;
    float attenuation_quadratic;
};

in vec3 normal;
in vec3 world_pos;
in vec2 texture_coord;

uniform vec3 camera_pos;
uniform Material material;
uniform PointLight point_lights[POINT_LIGHTS_NUM];
uniform SpotLight spot_light;

vec3 blinn_phong_ambient_term() {
    vec3 k_a = vec3(texture(material.diffuse_map, texture_coord));

    vec3 ambient_intensity = vec3(0.3, 0.3, 0.3);

    return k_a * ambient_intensity;
}

vec3 blinn_phong_diffuse_term(vec3 light_dir, vec3 light_intensity, vec3 n) {
    vec3 k_d = vec3(texture(material.diffuse_map, texture_coord));

    float cos_term = max(0.0, dot(n, light_dir));

    return k_d * light_intensity * cos_term;
}

vec3 blinn_phong_specular_term(vec3 light_dir, vec3 light_intensity, vec3 n, vec3 view_dir) {
    vec3 k_s = vec3(texture(material.specular_map, texture_coord));

    float p = 32.0;
    // vec3 reflect_vec = reflect(-light_dir, n);
    // float cos_term = pow(max(0.0, dot(view_dir, reflect_vec)), p); // Phong Model
    vec3 half_vec = normalize(light_dir + view_dir);
    float cos_term = pow(max(0.0, dot(n, half_vec)), p); // Blinn-Phong Model

    return k_s * light_intensity * cos_term;
}

vec3 calc_point_light(PointLight light, vec3 n, vec3 view_dir) {
    vec3 light_dir = normalize(light.position - world_pos);

    float light_distance = distance(light.position, world_pos);
    float attenuation = 1.0 / (1.0 + light.attenuation_linear * light_distance + light.attenuation_quadratic * (light_distance * light_distance));
    vec3 light_intensity = light.color * attenuation;

    vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light_intensity, n);
    vec3 specular_term = blinn_phong_specular_term(light_dir, light_intensity, n, view_dir);

    return diffuse_term + specular_term;
}

vec3 calc_spot_light(SpotLight light, vec3 n, vec3 view_dir) {
    vec3 light_dir = normalize(light.position - world_pos);

    float light_distance = distance(light.position, world_pos);
    float attenuation = 1.0 / (1.0 + light.attenuation_linear * light_distance + light.attenuation_quadratic * (light_distance * light_distance));
    float theta = dot(light_dir, normalize(-light.direction)); 
    float edge_smooth = clamp((theta - light.outer_cutoff) / (light.cutoff - light.outer_cutoff), 0.0, 1.0);
    vec3 light_intensity = light.color * attenuation * edge_smooth;

    vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light_intensity, n);
    vec3 specular_term = blinn_phong_specular_term(light_dir, light_intensity, n, view_dir);

    return diffuse_term + specular_term;
}

void main() {
    vec3 n = normalize(normal);
    vec3 view_dir = normalize(camera_pos - world_pos);

    vec3 rst;
    rst += blinn_phong_ambient_term();
    for(int i = 0; i < POINT_LIGHTS_NUM; i++)
        rst += calc_point_light(point_lights[i], n, view_dir);
    rst += calc_spot_light(spot_light, n, view_dir);

    frag_color = vec4(rst, 1.0);
}
