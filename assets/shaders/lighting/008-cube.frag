#version 330 core

out vec4 frag_color;

in vec3 normal;
in vec3 world_pos;

uniform vec3 object_color;
uniform vec3 light_color;
uniform vec3 light_pos;

vec3 calc_ambient_term() {
    vec3 ambient_intensity = vec3(0.1, 0.1, 0.1);

    vec3 k_a = object_color;

    return k_a * ambient_intensity;
}

vec3 calc_diffuse_term() {
    vec3 light_dir = normalize(light_pos - world_pos);
    float cos_term = max(0.0, dot(normal, light_dir));

    float r = distance(light_pos, world_pos);
    vec3 lighting_arrived = light_color / (r * r);

    vec3 k_d = object_color;

    return k_d * lighting_arrived * cos_term;
}

void main() {
    vec3 ambient_term = calc_ambient_term();
    vec3 diffuse_term = calc_diffuse_term();

    vec3 rst = ambient_term + diffuse_term;

    frag_color = vec4(rst, 1.0);
}
