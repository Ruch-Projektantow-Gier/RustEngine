#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Normal;
layout (location = 2) in vec2 TexCoords;
layout (location = 3) in vec3 Tangent;
layout (location = 4) in vec3 Bitangent;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

struct Light {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Light light;
uniform vec3 viewPos;

out VS_OUTPUT {
    vec2 TexCoords;
    vec3 FragPos;

    vec3 TangentLightPos;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
} OUT;

void main()
{
    gl_Position = projection * view * model * vec4(Position, 1.0);
    OUT.TexCoords = TexCoords;
    OUT.FragPos = vec3(model * vec4(Position, 1.0));

    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vec3 T = normalize(normalMatrix * Tangent);
    vec3 N = normalize(normalMatrix * Normal);
    T = normalize(T - dot(T, N) * N);
    vec3 B = cross(T, N);

    mat3 TBN = transpose(mat3(T, B, N));
    OUT.TangentLightPos = TBN * light.position;
    OUT.TangentViewPos  = TBN * viewPos;
    OUT.TangentFragPos  = TBN * OUT.FragPos;
}