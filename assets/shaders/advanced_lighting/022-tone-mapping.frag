#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

uniform sampler2D hdr_buffer;
uniform bool enable_tone_mapping;
uniform float exposure;

void main()
{
    const float gamma = 2.2;
    vec3 hdr_color = texture(hdr_buffer, texture_coord).rgb;

    if (enable_tone_mapping)
    {
        // Reinhard tone mapping
        // hdr_color = hdr_color / (hdr_color + vec3(1.0));
        // Exposure tone mapping
        hdr_color = vec3(1.0) - exp(-hdr_color * exposure);
    }

    vec3 rst = pow(hdr_color, vec3(1.0 / gamma)); // Gamma correct
    frag_color = vec4(rst, 1.0);
}
