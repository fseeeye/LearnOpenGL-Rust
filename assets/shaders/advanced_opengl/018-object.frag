#version 330 core

out vec4 frag_color;

in vec3 normal;
in vec3 world_pos;

uniform vec3 camera_pos;
uniform samplerCube skybox;

void main() {
    float ratio = 1.00 / 1.52;
    vec3 I = normalize(world_pos - camera_pos);
    // vec3 R = reflect(I, normalize(normal));
    vec3 R = refract(I, normalize(normal), ratio);
    frag_color = texture(skybox, R);
}
