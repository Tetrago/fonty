#version 330 core

layout(lines) in;

layout(triangle_strip, max_vertices = 8) out;

out vec2 g_Position;
out vec2 g_Point;

uniform mat4 u_Mvp;
uniform float u_Size;

const vec2 points[4] = vec2[4](
    vec2(-1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0)
);

void main()
{
    vec4 p0 = u_Mvp * gl_in[0].gl_Position;
    vec4 p1 = u_Mvp * gl_in[1].gl_Position;

    vec2 n0 = p0.xy / p0.w;
    vec2 n1 = p1.xy / p1.w;

    vec2 dir = normalize(n1 - n0);
    vec2 norm = vec2(-dir.y, dir.x);
    vec2 offset = norm * (u_Size * 0.5);

    for (int i = 0; i < 2; ++i)
    {
        vec4 p = (i == 0) ? p0 : p1;
        vec2 n = (i == 0) ? n0 : n1;

        vec2 a = n + offset;
        vec2 b = n - offset;

        vec4 clip_a = vec4(a * p.w, p.z, p.w);
        vec4 clip_b = vec4(b * p.w, p.z, p.w);

        gl_Position = clip_a;
        g_Position = a;
        g_Point = a;
        EmitVertex();

        gl_Position = clip_b;
        g_Position = b;
        g_Point = b;
        EmitVertex();
    }

    EndPrimitive();

    for (int i = 0; i < 4; ++i)
    {
        vec2 p = n0 + points[i] * u_Size * 0.5;
        vec4 point = vec4(p * p0.w, p0.z, p0.w);

        gl_Position = point;
        g_Position = n0;
        g_Point = p;
        EmitVertex();
    }

    EndPrimitive();
}
