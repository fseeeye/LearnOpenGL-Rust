#version 330 core

out float occlusion;

in vec2 texture_coord;

uniform sampler2D ssao;

void main()
{
    vec2 texel_size = 1.0 / vec2(textureSize(ssao, 0));
    float result = 0.0;
    // Use a 4x4 kernel to blur the ssao texture
    for (int x = -2; x < 2; ++x) 
    {
        for (int y = -2; y < 2; ++y) 
        {
            vec2 texcoord_offset = vec2(float(x), float(y)) * texel_size;
            // TODO: check if the texcoord is inside the texture
            result += texture(ssao, texture_coord + texcoord_offset).r;
        }
    }
    occlusion = result / (4.0 * 4.0);
}
