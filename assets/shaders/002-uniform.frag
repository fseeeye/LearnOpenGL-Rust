#version 330 core

out vec4 final_color;

uniform vec4 dyn_color;

void main() {
    final_color = dyn_color;
}