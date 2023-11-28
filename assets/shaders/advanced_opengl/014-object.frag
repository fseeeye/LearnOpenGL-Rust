#version 330 core

out vec4 frag_color;

struct Material {
    sampler2D diffuse_map;
};

in vec2 texture_coord;

uniform Material material;

void main() {
    frag_color = texture(material.diffuse_map, texture_coord);
}

