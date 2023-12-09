#version 330 core

out vec4 frag_color;

struct Material {
    sampler2D diffuse_map;
    // sampler2D specular_map;
    sampler2D normal_map;
    float shininess;
};

in VS_OUT {
    vec3 world_pos;
    vec2 texture_coord;
    mat3 TBN;
} fs_in;

uniform Material material;
uniform vec3 light_pos;
uniform vec3 camera_pos;

vec3 blinn_phong_ambient_term() {
    vec3 k_a = vec3(texture(material.diffuse_map, fs_in.texture_coord));

    vec3 ambient_intensity = vec3(0.3, 0.3, 0.3);

    return k_a * ambient_intensity;
}

vec3 blinn_phong_diffuse_term(vec3 light_dir, vec3 light_intensity, vec3 n) {
    vec3 k_d = vec3(texture(material.diffuse_map, fs_in.texture_coord));

    float cos_term = max(0.0, dot(n, light_dir));

    return k_d * light_intensity * cos_term;
}

vec3 blinn_phong_specular_term(vec3 light_dir, vec3 light_intensity, vec3 n, vec3 view_dir) {
    // vec3 k_s = vec3(texture(material.specular_map, texture_coord));
    vec3 k_s = vec3(texture(material.diffuse_map, fs_in.texture_coord));
    float p = material.shininess;

    vec3 half_vec = normalize(light_dir + view_dir);
    float cos_term = pow(max(0.0, dot(n, half_vec)), p); // Blinn-Phong Model

    return k_s * light_intensity * cos_term;
}

void main() {
    // Get normal from normal map
    vec3 normal = texture(material.normal_map, fs_in.texture_coord).rgb;
    // Convert normal from [0, 1] to [-1, 1]
    normal = normalize(normal * 2.0 - 1.0);
    // Apply TBN matrix to normal
    normal = normalize(fs_in.TBN * normal);
    
    vec3 view_dir = normalize(camera_pos - fs_in.world_pos);
    vec3 light_dir = normalize(light_pos - fs_in.world_pos);
    float light_distance = distance(light_pos, fs_in.world_pos);
    vec3 light_intensity = vec3(1.0);

    vec3 ambient_term = blinn_phong_ambient_term();
    vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light_intensity, normal);
    vec3 specular_term = blinn_phong_specular_term(light_dir, light_intensity, normal, view_dir);

    vec3 rst = ambient_term + diffuse_term + specular_term;
    frag_color = vec4(rst, 1.0);
    // frag_color = vec4(normal, 1.0);
}
