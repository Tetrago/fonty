#version 330 core

in vec2 g_Position;
in vec2 g_Point;

out vec4 f_Color;

uniform vec4 u_Color;
uniform float u_Size;

void main() {
    if (length(g_Point - g_Position) > u_Size * 0.5) {
        discard;
    }

    f_Color = u_Color;
}
