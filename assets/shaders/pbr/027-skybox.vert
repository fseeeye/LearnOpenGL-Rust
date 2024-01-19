#version 330 core

layout (location = 0) in vec3 a_pos;

out vec3 world_pos;

uniform mat4 projection;
uniform mat4 view;

void main()
{
    world_pos = a_pos;
    mat4 view_rot = mat4(mat3(view)); // remove translation from the view matrix
    vec4 clip_pos = projection * view_rot * vec4(world_pos, 1.0);
    gl_Position =  clip_pos.xyww; // keep z coordinate always at 1.0 (max depth)
}
