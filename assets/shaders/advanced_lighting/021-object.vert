#version 330 core

layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_texture_coord;
layout (location = 3) in vec3 a_tangent;
layout (location = 4) in vec3 a_bitangent;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat3 normal_matrix;
uniform vec3 light_pos;
uniform vec3 camera_pos;

out VS_OUT {
    vec3 world_pos;
    vec2 texture_coord;
    mat3 TBN_transpose;
    vec3 tangent_light_pos;
    vec3 tangent_view_pos;
    vec3 tangent_frag_pos;
} vs_out;

void main() {
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
    vs_out.world_pos = vec3(model * vec4(a_pos, 1.0));
    vs_out.texture_coord = a_texture_coord;
    // Calculate transposed TBN matrix for
    vec3 T = normalize(normal_matrix * a_tangent);
    vec3 B = normalize(normal_matrix * a_bitangent);
    vec3 N = normalize(normal_matrix * a_normal);
    vs_out.TBN_transpose = transpose(mat3(T, B, N));
    // Calculate light/view/fragment position in tangent space for parallax mapping
    vs_out.tangent_light_pos = vs_out.TBN_transpose * light_pos;
    vs_out.tangent_view_pos  = vs_out.TBN_transpose * camera_pos;
    vs_out.tangent_frag_pos  = vs_out.TBN_transpose * vs_out.world_pos;
}
