#version 330 core

layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat3 normal_matrix;

out vec3 normal;
out vec3 world_pos;

void main() {
    normal = normal_matrix * a_normal;
    world_pos = vec3(model * vec4(a_pos, 1.0));
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
}
