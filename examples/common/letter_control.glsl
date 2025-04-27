#version 400 core

layout(vertices = 3) out;

void main() {
    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

    if (gl_InvocationID == 0)
    {
        gl_TessLevelOuter[0] = 1.0;

        vec2 a = (gl_out[1].gl_Position - gl_out[0].gl_Position).xy;
        vec2 b = (gl_out[2].gl_Position - gl_out[1].gl_Position).xy;
        gl_TessLevelOuter[1] = a == b ? 1.0 : 16.0;
    }
}
