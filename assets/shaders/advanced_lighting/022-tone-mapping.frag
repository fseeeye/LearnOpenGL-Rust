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

    if(enable_tone_mapping)
    {
        // Reinhard tone mapping
        // vec3 rst = hdr_color / (hdr_color + vec3(1.0));
        // Exposure
        vec3 rst = vec3(1.0) - exp(-hdr_color * exposure);

        // Gamma correct
        rst = pow(rst, vec3(1.0 / gamma));

        frag_color = vec4(rst, 1.0);
    }
    else
    {
        // Gamma correct
        vec3 rst = pow(hdr_color, vec3(1.0 / gamma));

        frag_color = vec4(rst, 1.0);
    }
}
