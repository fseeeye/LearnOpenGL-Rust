#version 330 core

layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_texture_coord; // texture coord from vertex attribute

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform bool normal_inverted;

out VS_OUT {
    vec3 frag_pos_view;
    vec3 normal;
    vec2 texture_coord;
} vs_out;

void main() {
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
    vs_out.frag_pos_view = vec3(view * model * vec4(a_pos, 1.0));
    vs_out.texture_coord = a_texture_coord;
    mat3 normal_matrix = transpose(inverse(mat3(view * model)));
    vs_out.normal = normal_matrix * ((normal_inverted ? -a_normal : a_normal));
}
