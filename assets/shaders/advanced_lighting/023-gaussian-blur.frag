#version 330 core

out vec4 frag_color;

in vec2 texture_coord;

uniform sampler2D image;
uniform bool horizontal_blur;
uniform float weight[5] = float[] (0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main() {
    vec2 texel_offset = 1.0 / textureSize(image, 0);
    vec3 rst = texture(image, texture_coord).rgb  * weight[0]; // init result as current pixel's contribution
    if (horizontal_blur) {
        for (int i = 1; i < 5; i++) {
            rst += texture(image, texture_coord + vec2(texel_offset.x * i, 0.0)).rgb * weight[i];
            rst += texture(image, texture_coord - vec2(texel_offset.x * i, 0.0)).rgb * weight[i];
        }
    } else {
        for (int i = 1; i < 5; i++) {
            rst += texture(image, texture_coord + vec2(0.0, texel_offset.y * i)).rgb * weight[i];
            rst += texture(image, texture_coord - vec2(0.0, texel_offset.y * i)).rgb * weight[i];
        }
    }

    frag_color = vec4(rst, 1.0);
}
