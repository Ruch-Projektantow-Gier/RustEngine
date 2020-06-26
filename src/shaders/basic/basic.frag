#version 330 core

// Material
struct Material {
    sampler2D texture_diffuse1;
//    sampler2D texture_specular1;
    sampler2D texture_normal1;

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
    vec3 FragPos;

    vec3 TangentLightPos;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
} IN;

out vec4 FragColor;

void main()
{
    vec3 normal = texture(material.texture_normal1, IN.TexCoords).rgb;
    normal = normalize(normal * 2.0 - 1.0);

    // ambient
    vec3 ambient  = light.ambient * vec3(texture(material.texture_diffuse1, IN.TexCoords));

    // diffuse
    vec3 lightDir = normalize(IN.TangentLightPos - IN.TangentFragPos);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.texture_diffuse1, IN.TexCoords));

    // specular
    vec3 viewDir = normalize(IN.TangentViewPos - IN.TangentFragPos);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
//    vec3 specular = light.specular * spec * vec3(texture(material.texture_specular1, IN.TexCoords));
    vec3 specular = light.specular * spec;

    FragColor = vec4(ambient + diffuse + specular, texture(material.texture_diffuse1, IN.TexCoords).a);
}
