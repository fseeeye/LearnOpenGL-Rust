#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

uniform sampler2D hdr_buffer;
uniform sampler2D bloom_blur_buffer;
uniform bool enable_bloom;
uniform float exposure;

void main()
{
    const float gamma = 2.2;
    vec3 hdr_color = texture(hdr_buffer, texture_coord).rgb;
    vec3 bloom_color = texture(bloom_blur_buffer, texture_coord).rgb;

    vec3 rst = vec3(0.0);
    // Reinhard tone mapping
    // rst += hdr_color / (hdr_color + vec3(1.0));
    // Exposure tone mapping
    rst += vec3(1.0) - exp(-hdr_color * exposure);

    if (enable_bloom)
    {
        rst += bloom_color;
    }

    rst = pow(rst, vec3(1.0 / gamma)); // Gamma correct
    frag_color = vec4(rst, 1.0);
}
