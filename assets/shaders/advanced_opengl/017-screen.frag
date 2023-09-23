#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

uniform sampler2D screen_texture;

void main() {
    frag_color = vec4(texture(screen_texture, texture_coord).rgb, 1.0);
}
