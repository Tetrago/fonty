#version 400 core

layout(isolines, equal_spacing) in;

void main()
{
    float u = gl_TessCoord.x;

    vec4 a = gl_in[0].gl_Position;
    vec4 b = gl_in[1].gl_Position;
    vec4 c = gl_in[2].gl_Position;

    gl_Position = (a * (1 - u) + 2 * b * u) * (1 - u) + c * u * u;
}
