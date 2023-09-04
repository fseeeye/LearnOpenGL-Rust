#version 330 core

out vec4 frag_color;

in vec3 normal;
in vec3 world_pos;
in vec2 texture_coord;

// uniform vec3 light_color;
// uniform vec3 light_pos;
uniform vec3 camera_pos;

struct Material {
    sampler2D diffuse_map;
    sampler2D specular_map;
    float shininess;
}; 
uniform Material material;

struct DirLight {
    vec3 direction;
    vec3 color;
};  
uniform DirLight dir_light;

struct PointLight {
    vec3 position;
    vec3 color;

    float attenuation_linear;
    float attenuation_quadratic;
};  
#define POINT_LIGHTS_NUM 4
uniform PointLight pointLights[POINT_LIGHTS_NUM];

vec3 blinn_phong_ambient_term() {
    vec3 k_a = vec3(texture(material.diffuse_map, texture_coord));

    vec3 ambient_intensity = vec3(0.2, 0.2, 0.2);

    return k_a * ambient_intensity;
}

vec3 blinn_phong_diffuse_term(vec3 light_dir, vec3 light, vec3 n) {
    vec3 k_d = vec3(texture(material.diffuse_map, texture_coord));

    float cos_term = max(0.0, dot(n, light_dir));

    return k_d * light * cos_term;
}

vec3 blinn_phong_specular_term(vec3 light_dir, vec3 light, vec3 n, vec3 view_dir) {
    vec3 k_s = vec3(texture(material.specular_map, texture_coord));

    float p = material.shininess;
    // vec3 reflect_vec = reflect(-light_dir, n);
    // float cos_term = pow(max(0.0, dot(view_dir, reflect_vec)), p); // Phong Model
    vec3 half_vec = normalize(light_dir + view_dir);
    float cos_term = pow(max(0.0, dot(n, half_vec)), p); // Blinn-Phong Model

    return k_s * light * cos_term;
}

vec3 calc_dir_light(DirLight light, vec3 n, vec3 view_dir) {
    vec3 light_dir = normalize(-light.direction);

    vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light.color, n);
    vec3 specular_term = blinn_phong_specular_term(light_dir, light.color, n, view_dir);

    return diffuse_term + specular_term;
}

vec3 calc_point_light(PointLight light, vec3 n, vec3 view_dir) {
    vec3 light_dir = normalize(light.position - world_pos);

    float light_distance = distance(light.position, world_pos);
    float attenuation = 1.0 / (1.0 + light.attenuation_linear * light_distance + light.attenuation_quadratic * (light_distance * light_distance));
    vec3 light_arrived = light.color / attenuation;

    vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light_arrived, n);
    vec3 specular_term = blinn_phong_specular_term(light_dir, light_arrived, n, view_dir);

    return diffuse_term + specular_term;
}

void main() {
    vec3 n = normalize(normal);
    vec3 view_dir = normalize(camera_pos - world_pos);

    vec3 rst;
    rst += blinn_phong_ambient_term();
    rst += calc_dir_light(dir_light, n, view_dir);
    for(int i = 0; i < POINT_LIGHTS_NUM; i++)
        rst += calc_point_light(pointLights[i], n, view_dir);

    frag_color = vec4(rst, 1.0);
}
