#version 330 core

layout(location = 0) in vec2 i_Position;
layout(location = 1) in int i_Curve;

out int v_Curve;

void main() {
    gl_Position = vec4(i_Position, 0.0, 1.0);
    v_Curve = i_Curve;
}
