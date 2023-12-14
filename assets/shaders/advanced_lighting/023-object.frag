#version 330 core

layout (location = 0) out vec4 frag_color;
layout (location = 1) out vec4 bright_color;

struct Material {
    sampler2D diffuse_map;
    // sampler2D specular_map;
    // sampler2D normal_map;
    float shininess;
};

struct Light {
    vec3 position;
    vec3 color;
};

in VS_OUT {
    vec3 world_pos;
    vec3 normal;
    vec2 texture_coord;
} fs_in;

uniform Material material;
uniform Light lights[16];

vec3 blinn_phong_diffuse_term(vec3 light_dir, vec3 light_intensity, vec3 n) {
    vec3 k_d = vec3(texture(material.diffuse_map, fs_in.texture_coord));

    float cos_term = max(0.0, dot(n, light_dir));

    return k_d * light_intensity * cos_term;
}

void main() {
    vec3 n = normalize(fs_in.normal);

    /* Calculate fragment color */
    vec3 rst = vec3(0.0);
    for(int i = 0; i < 4; i++) {
        vec3  light_dir = normalize(lights[i].position - fs_in.world_pos);
        float light_distance = distance(lights[i].position, fs_in.world_pos);
        vec3  light_intensity = lights[i].color;

        // Calculate diffuse term
        vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light_intensity, n);

        // Lighting attenuation
        // use quadratic attenuation as we have gamma correction in tone mapping shader.
        diffuse_term *= 1.0 / (light_distance * light_distance);

        rst += diffuse_term;
    }
    frag_color = vec4(rst, 1.0);

    /* Calculate brightness color */
    float brightness = dot(frag_color.rgb, vec3(0.2126, 0.7152, 0.0722));
    // Check whether fragment output is higher than threshold, if so output as brightness color
    if (brightness > 1.0)
        bright_color = vec4(frag_color.rgb, 1.0);
    else
        bright_color = vec4(0.0, 0.0, 0.0, 1.0);
}
