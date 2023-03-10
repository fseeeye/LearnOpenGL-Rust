#version 330 core

layout (location = 0) in vec3 a_pos;

out vec4 vertex_color;

void main() {
    vertex_color = vec4(1.0, 0.5, 0.2, 1.0);

    gl_Position = vec4(a_pos.x, a_pos.y, a_pos.z, 1.0);
}