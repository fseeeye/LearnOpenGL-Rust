#version 330 core

layout (location = 0) out vec4 frag_color;
layout (location = 1) out vec4 bright_color;

in VS_OUT {
    vec3 world_pos;
    vec3 normal;
    vec2 texture_coord;
} fs_in;

uniform vec3 light_color;

void main() {
    frag_color = vec4(light_color, 1.0);

    /* Calculate brightness color */
    float brightness = dot(frag_color.rgb, vec3(0.2126, 0.7152, 0.0722));
    // Check whether fragment output is higher than threshold, if so output as brightness color
    if (brightness > 1.0)
        bright_color = vec4(frag_color.rgb, 1.0);
    else
        bright_color = vec4(0.0, 0.0, 0.0, 1.0);
}
