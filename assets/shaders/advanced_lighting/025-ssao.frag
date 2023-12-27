#version 330 core

out float occlusion;

in vec2 texture_coord;

// Parameters (you'd probably want to use them as uniforms to more easily tweak the effect)
const int sample_kernel_size = 64;
// Tile noise texture over screen based on screen dimensions divided by noise size
const vec2 noise_scale = vec2(800.0 / 4.0, 600.0 / 4.0);

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform sampler2D ssao_noise;
uniform mat4 projection;
uniform vec3 ssao_samples[64];
uniform float ssao_radius; // radius value to control sample kernel radius
uniform float ssao_bias;

void main()
{
    // Get input for SSAO algorithm
    vec3 frag_pos_view = texture(g_position, texture_coord).xyz;
    vec3 normal = normalize(texture(g_normal, texture_coord).rgb);
    vec3 noise_vec = normalize(texture(ssao_noise, texture_coord * noise_scale).xyz);

    // Create TBN matrix by Gramm-Schmidt Process : from tangent space to view space
    vec3 tangent = normalize(noise_vec - normal * dot(noise_vec, normal));
    vec3 bitangent = cross(normal, tangent);
    mat3 TBN = mat3(tangent, bitangent, normal);

    // Iterate over the sample kernel and calculate occlusion factor
    float occlusion_times = 0.0;
    for(int i = 0; i < sample_kernel_size; ++i)
    {
        // Get sample position from tangent space to view space
        vec3 sample_pos = frag_pos_view + (TBN * ssao_samples[i]) * ssao_radius;

        // Project sample position to sample depth on g_position texture
        vec4 sample_pos_clip = projection * vec4(sample_pos, 1.0); // from view space to clip space
        sample_pos_clip.xyz /= sample_pos_clip.w; // perspective divide
        sample_pos_clip.xyz = sample_pos_clip.xyz * 0.5 + 0.5; // transform to range 0.0 - 1.0

        // Get sample depth
        float sample_depth = texture(g_position, sample_pos_clip.xy).z; // get depth value of kernel sample

        // Range check
        // float range_check = 1.0;
        float range_check = smoothstep(0.0, 1.0, ssao_radius / abs(frag_pos_view.z - sample_depth));

        // Calculate occlusion factor
        occlusion_times += (sample_depth >= sample_pos.z + ssao_bias ? 1.0 : 0.0) * range_check;
    }

    occlusion = 1.0 - (occlusion_times / sample_kernel_size);
}
