#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

uniform sampler2D shadow_map;

void main()
{
    float depth = texture(shadow_map, texture_coord).r;
    frag_color = vec4(vec3(depth), 1.0); // orthographic
}
