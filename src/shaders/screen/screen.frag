#version 330 core

uniform sampler2D texture_diffuse1;

in VS_OUTPUT {
    vec2 TexCoords;
} IN;

out vec4 FragColor;

void main()
{
    FragColor = texture(texture_diffuse1, IN.TexCoords);
}