#version 330 core

uniform sampler2D texture_diffuse1;
uniform vec3 lightPos;
uniform vec3 lightColor;
uniform vec3 viewPos;

in VS_OUTPUT {
    vec2 TexCoords;
    vec3 Normal;
    vec3 FragPos;
} IN;

out vec4 FragColor;

void main()
{
    // ambient
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;

    // diffuse
    vec3 norm = normalize(IN.Normal);
    vec3 lightDir = normalize(lightPos - IN.FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    // specular
    float specularStrength = 0.5;
    vec3 viewDir = normalize(viewPos - IN.FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
    vec3 specular = specularStrength * spec * lightColor;

    vec3 result = (ambient + diffuse + specular) * texture(texture_diffuse1, IN.TexCoords).rgb;
    FragColor = vec4(result, texture(texture_diffuse1, IN.TexCoords).a);
}