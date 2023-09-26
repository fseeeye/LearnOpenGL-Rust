#version 330 core

layout (location = 0) in vec3 a_pos;

uniform mat4 view;
uniform mat4 projection;

out vec3 texture_coord;

void main() {
    texture_coord = normalize(a_pos);
    vec4 pos = projection * view * vec4(a_pos, 1.0);
    gl_Position = pos.xyww; // let depth of skybox always be 1.0
}
