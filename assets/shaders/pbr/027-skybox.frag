#version 330 core

out vec4 frag_color;

in vec3 world_pos;

uniform samplerCube environment_map;

void main()
{
    // vec3 env_color = texture(environment_map, normalize(world_pos)).rgb;
    vec3 env_color = textureLod(environment_map, normalize(world_pos), 1.2).rgb;

    env_color = env_color / (env_color + vec3(1.0)); // tone mapping
    env_color = pow(env_color, vec3(1.0/2.2)); // gamma correction

    frag_color = vec4(env_color, 1.0);
}