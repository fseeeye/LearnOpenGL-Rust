#version 330 core

in vec2 texture_coord; // texture coord from vertex shader

out vec4 frag_color;

uniform sampler2D t_container; // 2D texture sampler

void main() {
    frag_color = texture(t_container, texture_coord);
}