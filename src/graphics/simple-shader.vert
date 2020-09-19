#version 140

in vec3 position;
in vec4 color;

out vec4 in_color;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
    in_color = color;
    gl_Position = perspective * view * model * vec4(position, 1.0);
}