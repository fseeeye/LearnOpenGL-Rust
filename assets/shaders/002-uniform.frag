#version 330 core

out vec4 frag_color;

uniform vec4 dyn_color;

void main() {
    frag_color = dyn_color;
}