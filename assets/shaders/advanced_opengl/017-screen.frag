#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

uniform sampler2D screen_texture;

void main() {
    frag_color = vec4(texture(screen_texture, texture_coord).rgb, 1.0);
    float average = frag_color.r * 0.299 + frag_color.g * 0.587 + frag_color.b * 0.114;
    frag_color = vec4(average, average, average, 1.0);
}
