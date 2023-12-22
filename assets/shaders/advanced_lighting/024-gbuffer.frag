#version 330 core

layout (location = 0) out vec3 g_position;
layout (location = 1) out vec3 g_normal;
layout (location = 2) out vec4 g_albedo_spec;

struct Material {
    sampler2D diffuse_map;
    sampler2D specular_map;
    // sampler2D normal_map;
    // float shininess;
};

in VS_OUT {
    vec3 world_pos;
    vec3 normal;
    vec2 texture_coord;
} fs_in;

uniform Material material;

void main() {
    // Store the fragment position vector
    g_position = fs_in.world_pos;
    // Store the per-fragment normals
    g_normal = normalize(fs_in.normal);
    // Store the diffuse per-fragment color
    g_albedo_spec.rgb = texture(material.diffuse_map, fs_in.texture_coord).rgb;
    // Store specular intensity in gAlbedoSpec's alpha component
    g_albedo_spec.a = texture(material.specular_map, fs_in.texture_coord).r;
}
