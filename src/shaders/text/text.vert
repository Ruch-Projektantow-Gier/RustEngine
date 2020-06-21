#version 330 core

layout (location = 0) in vec4 PosAndTexCoord;

uniform mat4 projection;

out VS_OUTPUT {
    vec2 TexCoords;
} OUT;


void main()
{
    gl_Position = projection * vec4(PosAndTexCoord.xy, 0.0, 1.0);
    OUT.TexCoords = PosAndTexCoord.zw;
}