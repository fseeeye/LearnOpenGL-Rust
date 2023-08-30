#version 330 core

layout (location = 0) in vec3 a_pos;
layout (location = 0) in vec3 a_normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 normal;
out vec3 world_pos;

void main() {
    normal = a_normal;
    world_pos = vec3(model * vec4(a_pos, 1.0));
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
}
