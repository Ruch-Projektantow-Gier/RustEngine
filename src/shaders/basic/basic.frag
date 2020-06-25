#version 330 core

uniform sampler2D texture_diffuse1;
uniform vec3 viewPos;

// Material
struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

uniform Material material;

// Light
struct Light {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Light light;

in VS_OUTPUT {
    vec2 TexCoords;
    vec3 Normal;
    vec3 FragPos;
} IN;

out vec4 FragColor;

void main()
{
    // ambient
    vec3 ambient  = light.ambient * material.ambient;

    // diffuse
    vec3 norm = normalize(IN.Normal);
    vec3 lightDir = normalize(light.position - IN.FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * (diff * material.diffuse);

    // specular
    vec3 viewDir = normalize(viewPos - IN.FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * (spec * material.specular);

     vec3 result = (ambient + diffuse + specular) * texture(texture_diffuse1, IN.TexCoords).rgb;
    FragColor = vec4(result, texture(texture_diffuse1, IN.TexCoords).a);
    //    vec3 result = (ambient + diffuse + specular);
//    FragColor = vec4(result, 1.0);
}