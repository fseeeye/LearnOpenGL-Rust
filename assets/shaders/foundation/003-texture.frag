#version 330 core

in vec2 texture_coord; // texture coord from vertex shader

out vec4 frag_color;

uniform sampler2D t_container; // 2D texture sampler
uniform sampler2D t_face; // 2D texture sampler

void main() {
    frag_color = mix(texture(t_container, texture_coord), texture(t_face, texture_coord), 0.2);
}