#version 330 core

uniform sampler2D ourTexture;

in VS_OUTPUT {
    vec3 Color;
    vec2 TexCoords;
} IN;

out vec4 Color;

void main()
{
    Color = vec4(mix(texture(ourTexture, IN.TexCoords).rgb, IN.Color, 0.5), 1.0);
}