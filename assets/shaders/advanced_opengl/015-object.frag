#version 330 core

out vec4 frag_color;

struct Material {
    sampler2D diffuse_map;
};

in vec2 texture_coord;

uniform Material material;

void main() {
    // frag_color = vec4(vec3(gl_FragCoord.z), 1.0);
    vec4 texture_color = texture(material.diffuse_map, texture_coord);
    if(texture_color.a < 0.1)
        discard;
    frag_color = texture_color;
}
