#version 400 core

layout(vertices = 3) out;

in int v_Curve[];

void main() {
    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

    if (gl_InvocationID == 0)
    {
        gl_TessLevelOuter[0] = 1.0;
        gl_TessLevelOuter[1] = v_Curve[1] != 0 ? 1.0 : 16.0;
    }
}
