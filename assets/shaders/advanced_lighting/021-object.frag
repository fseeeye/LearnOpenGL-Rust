#version 330 core

out vec4 frag_color;

struct Material {
    sampler2D diffuse_map;
    // sampler2D specular_map;
    sampler2D normal_map;
    float shininess;
    sampler2D displacement_map;
    float depth_scale;
};

in VS_OUT {
    vec3 world_pos;
    vec2 texture_coord;
    mat3 TBN_transpose;
    vec3 tangent_light_pos;
    vec3 tangent_view_pos;
    vec3 tangent_frag_pos;
} fs_in;

uniform Material material;
uniform vec3 light_pos;
uniform vec3 camera_pos;

vec3 blinn_phong_ambient_term(vec2 texture_coord) {
    vec3 k_a = vec3(texture(material.diffuse_map, texture_coord));

    vec3 ambient_intensity = vec3(0.3, 0.3, 0.3);

    return k_a * ambient_intensity;
}

vec3 blinn_phong_diffuse_term(vec3 light_dir, vec3 light_intensity, vec3 n, vec2 texture_coord) {
    vec3 k_d = vec3(texture(material.diffuse_map, texture_coord));

    float cos_term = max(0.0, dot(n, light_dir));

    return k_d * light_intensity * cos_term;
}

vec3 blinn_phong_specular_term(vec3 light_dir, vec3 light_intensity, vec3 n, vec3 view_dir, vec2 texture_coord) {
    // vec3 k_s = vec3(texture(material.specular_map, texture_coord));
    vec3 k_s = vec3(texture(material.diffuse_map, texture_coord));
    float p = material.shininess;

    vec3 half_vec = normalize(light_dir + view_dir);
    float cos_term = pow(max(0.0, dot(n, half_vec)), p); // Blinn-Phong Model

    return k_s * light_intensity * cos_term;
}

vec2 parallax_mapping(vec2 texture_coord, vec3 view_dir)
{
    /* Traditional Parallax Mapping */
    // float depth = texture(material.displacement_map, texture_coord).r;
    // vec2 p_texcoords_vector = -(view_dir.xy / view_dir.z * (depth * material.depth_scale));
    // return texture_coord + p_texcoords_vector;

    /* Steep Parallax Mapping */
    // get initial values
    vec2 current_texture_coords = texture_coord;
    float current_depth = texture(material.displacement_map, current_texture_coords).r;
    float current_layer_depth = 0.0;
    // number of depth layers
    const float min_layers = 8;
    const float max_layers = 32;
    float layers_num = mix(max_layers, min_layers, abs(dot(vec3(0.0, 0.0, 1.0), view_dir))); 
    // calculate the size of each layer
    float layer_depth_step = 1.0 / layers_num;
    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 p_texcoords_vector = view_dir.xy / view_dir.z * material.depth_scale;
    vec2 texture_coords_delta = p_texcoords_vector / layers_num;
    // Traverse depth layers
    for (int i = 0; i < layers_num; i++)
    {
        if (current_layer_depth >= current_depth) {
            break;
        }
        // shift texture coordinates along direction of P
        current_texture_coords -= texture_coords_delta;
        // get depthmap value at current texture coordinates
        current_depth = texture(material.displacement_map, current_texture_coords).r;  
        // get depth of next layer
        current_layer_depth += layer_depth_step;
    }

    /* Parallax Occlusion Mapping */
    // get texture coordinates before collision (reverse operations)
    vec2 prev_texture_coords = current_texture_coords + texture_coords_delta;
    // get depth after and before collision for linear interpolation
    float after_depth = current_layer_depth - current_depth;
    float before_depth = 
        texture(material.displacement_map, prev_texture_coords).r - (current_layer_depth - layer_depth_step);
    // interpolation of texture coordinates
    float weight = after_depth / (after_depth + before_depth);
    vec2 final_texture_coords = prev_texture_coords * weight + current_texture_coords * (1.0 - weight);

    // return current_texture_coords; // Steep Parallax Mapping
    return final_texture_coords; // Parallax Occlusion Mapping
}

void main() {
    // Calculate light/view dir in tangent space
    vec3 light_dir = normalize(fs_in.tangent_light_pos - fs_in.tangent_frag_pos);
    vec3 view_dir = normalize(fs_in.tangent_view_pos - fs_in.tangent_frag_pos);

    // Calculate offset texture coordinates with Parallax Mapping
    vec2 texture_coord = parallax_mapping(fs_in.texture_coord, view_dir);
    if(texture_coord.x > 1.0 || texture_coord.y > 1.0 || texture_coord.x < 0.0 || texture_coord.y < 0.0)
        discard; // Discard fragments outside of texture boundaries

    // Calculate normal in tangent space
    vec3 normal = texture(material.normal_map, texture_coord).rgb;
    normal = normalize(normal * 2.0 - 1.0);
    // normal = normalize(fs_in.TBN * normal);

    float light_distance = distance(light_pos, fs_in.world_pos);
    vec3 light_intensity = vec3(1.0);

    vec3 ambient_term = blinn_phong_ambient_term(texture_coord);
    vec3 diffuse_term = blinn_phong_diffuse_term(light_dir, light_intensity, normal, texture_coord);
    vec3 specular_term = blinn_phong_specular_term(light_dir, light_intensity, normal, view_dir, texture_coord);
    frag_color = vec4(ambient_term + diffuse_term + specular_term, 1.0);
}
