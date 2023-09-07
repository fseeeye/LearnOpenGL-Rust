#version 330 core

out vec4 frag_color;

struct Material {
    sampler2D diffuse_map;
    sampler2D specular_map;
    sampler2D normal_map;
    float shininess;
};

in vec3 normal;
in vec3 world_pos;
in vec2 texture_coord;

uniform vec3 camera_pos;
uniform Material material;

void main() {
    frag_color = texture(material.diffuse_map, texture_coord);
}
