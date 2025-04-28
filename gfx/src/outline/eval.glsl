#version 400 core

layout(isolines, equal_spacing) in;

vec4 lerp(vec4 from, vec4 to, float u)
{
    return from * (1 - u) + to * u;
}

void main()
{
    float u = gl_TessCoord.x;

    vec4 a = gl_in[0].gl_Position;
    vec4 b = gl_in[1].gl_Position;
    vec4 c = gl_in[2].gl_Position;

    vec4 d = lerp(a, b, u);
    vec4 e = lerp(b, c, u);

    gl_Position = lerp(d, e, u);
}
