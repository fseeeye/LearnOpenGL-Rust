#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

struct Light {
    vec3 position;
    vec3 color;
};

uniform Light light;
// uniform vec3 camera_pos;
uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform sampler2D g_albedo_spec;
uniform sampler2D ssao;
uniform bool enable_ssao;

void main()
{
    // Get data from GBuffer
    vec3 frag_pos_view = texture(g_position, texture_coord).rgb;
    vec3 normal = texture(g_normal, texture_coord).rgb;
    const float k_a = 0.1;
    vec3 k_d = texture(g_albedo_spec, texture_coord).rgb;
    float k_s = texture(g_albedo_spec, texture_coord).a;
    float ambient_occlusion = texture(ssao, texture_coord).r;

    /* Calculate lighting */

    vec3 view_dir = normalize(-frag_pos_view); // camera pos is (0, 0, 0)
    vec3 light_dir = normalize(light.position - frag_pos_view);
    // Calculate ambient
    vec3 ambient_term;
    if (enable_ssao)
        ambient_term = vec3(k_a * k_d * ambient_occlusion);
    else 
        ambient_term = vec3(k_a * k_d);
    // Calculate diffuse
    vec3 diffuse_term = k_d * light.color * max(0.0, dot(normal, light_dir));
    // Calculate specular (Blinn-Phong Model)
    vec3 half_vec = normalize(light_dir + view_dir);
    vec3 specular_term = k_s * light.color * pow(max(0.0, dot(normal, half_vec)), 16.0);
    // Lighting attenuation
    float light_distance = distance(light.position, frag_pos_view);
    float attenuation = 1.0 / (light_distance * light_distance);

    vec3 rst = ambient_term + diffuse_term * attenuation + specular_term * attenuation;
    rst = pow(rst, vec3(1.0 / 2.2)); // Gamma correct

    frag_color = vec4(rst, 1.0);
}
