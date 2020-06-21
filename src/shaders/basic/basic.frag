#version 330 core

uniform sampler2D texture_diffuse1;
uniform vec3 light_pos;

in VS_OUTPUT {
    vec3 Color;
    vec2 TexCoords;
    vec3 Normal;
} IN;

out vec4 Color;

void main()
{
    Color = vec4(mix(texture(texture_diffuse1, IN.TexCoords).rgb, IN.Color, 0.5), 1.0);
//    Color = vec4(IN.Color, 1.0);
}