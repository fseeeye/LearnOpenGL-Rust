#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

struct Light {
    vec3 position;
    vec3 color;
};

const int LIGHT_NUM = 32;
uniform Light lights[LIGHT_NUM];
uniform vec3 camera_pos;
uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform sampler2D g_albedo_spec;

void main()
{
    // Get data from GBuffer
    vec3 world_pos = texture(g_position, texture_coord).rgb;
    vec3 normal = texture(g_normal, texture_coord).rgb;
    float k_a = 0.1;
    vec3 k_d = texture(g_albedo_spec, texture_coord).rgb;
    float k_s = texture(g_albedo_spec, texture_coord).a;

    /* Calculate lighting */

    vec3 view_dir = normalize(camera_pos - world_pos);
    // Calculate ambient
    vec3 rst = k_a * k_d;
    for (int i = 0; i < LIGHT_NUM; i++)
    {
        vec3 light_dir = normalize(lights[i].position - world_pos);
        // Calculate diffuse
        vec3 diffuse_term = k_d * lights[i].color * max(0.0, dot(normal, light_dir));
        // Calculate specular (Blinn-Phong Model)
        vec3 half_vec = normalize(light_dir + view_dir);
        vec3 specular_term = k_s * lights[i].color * pow(max(0.0, dot(normal, half_vec)), 16.0);
        // Lighting attenuation
        float light_distance = distance(lights[i].position, world_pos);
        float attenuation = 1.0 / (light_distance * light_distance);

        rst += diffuse_term * attenuation + specular_term * attenuation;
    }

    frag_color = vec4(rst, 1.0);
}
