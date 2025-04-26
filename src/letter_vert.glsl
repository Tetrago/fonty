#version 330 core

layout(location = 0) in vec2 i_Position;

uniform mat4 u_Mvp;

void main() {
    gl_Position = u_Mvp * vec4(i_Position, 0.0, 1.0);
}
