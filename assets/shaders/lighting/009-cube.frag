#version 330 core

out vec4 frag_color;

in vec3 normal;
in vec3 world_pos;
in vec2 texture_coord;

uniform vec3 light_color;
uniform vec3 light_pos;
uniform vec3 camera_pos;

struct Material {
    sampler2D diffuse_map;
    sampler2D specular_map;
    float shininess;
    sampler2D emission_map;
}; 
uniform Material material;

vec3 calc_ambient_term() {
    vec3 k_a = vec3(texture(material.diffuse_map, texture_coord));

    vec3 ambient_intensity = vec3(0.2, 0.2, 0.2);

    return k_a * ambient_intensity;
}

vec3 calc_diffuse_term() {
    vec3 k_d = vec3(texture(material.diffuse_map, texture_coord));

    vec3 light_dir = normalize(light_pos - world_pos);
    float cos_term = max(0.0, dot(normal, light_dir));

    float r = distance(light_pos, world_pos);
    vec3 lighting_arrived = light_color / (r * r);

    return k_d * lighting_arrived * cos_term;
}

vec3 calc_specular_term() {
    vec3 k_s = vec3(texture(material.specular_map, texture_coord));

    float p = material.shininess;
    vec3 light_dir = normalize(light_pos - world_pos);
    vec3 view_dir = normalize(camera_pos - world_pos);
    // vec3 reflect_vec = reflect(-light_dir, normal);
    // float cos_term = pow(max(0.0, dot(view_dir, reflect_vec)), p); // Phong Model
    vec3 half_vec = normalize(light_dir + view_dir);
    float cos_term = pow(max(0.0, dot(normal, half_vec)), p); // Blinn-Phong Model

    return k_s * light_color * cos_term;
}

void main() {
    vec3 ambient_term = calc_ambient_term();
    vec3 diffuse_term = calc_diffuse_term();
    vec3 specular_term = calc_specular_term();

    vec3 emission = texture(material.emission_map, texture_coord).rgb * 0.2;

    vec3 rst = ambient_term + diffuse_term + specular_term + emission;

    frag_color = vec4(rst, 1.0);
}
