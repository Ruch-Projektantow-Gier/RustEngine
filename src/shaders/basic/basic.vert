#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Normal;
layout (location = 2) in vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out VS_OUTPUT {
    vec2 TexCoords;
    vec3 Normal;
    vec3 FragPos;
} OUT;

void main()
{
    gl_Position = projection * view * model * vec4(Position, 1.0);
    OUT.TexCoords = TexCoords;
    OUT.Normal = mat3(transpose(inverse(model))) * Normal;
    OUT.FragPos = vec3(model * vec4(Position, 1.0));;
}