#version 330 core

layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_texture_coord; // texture coord from vertex attribute

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
// uniform mat3 normal_matrix;

// out vec3 normal;
out vec3 world_pos;
out vec2 texture_coord;

void main() {
    // normal = normal_matrix * a_normal;
    world_pos = vec3(model * vec4(a_pos, 1.0));
    texture_coord = a_texture_coord;
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
}
