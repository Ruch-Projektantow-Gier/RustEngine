#version 330 core

uniform sampler2D text;
uniform vec3 textColor;

in VS_OUTPUT {
    vec2 TexCoords;
} IN;

out vec4 FragColor;

void main()
{
    vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text, IN.TexCoords).r);
    FragColor = vec4(textColor, 1.0) * sampled;
}