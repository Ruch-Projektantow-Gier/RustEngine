#version 330 core

// Material
struct Material {
    sampler2D texture_diffuse1;
//    sampler2D texture_specular1;
    sampler2D texture_normal1;
    sampler2D texture_height1;

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

uniform float height_scale;

vec2 ParallaxMapping(vec2 texCoords, vec3 viewDir)
{
    float height =  texture(material.texture_height1, texCoords).r;
    vec2 p = viewDir.xy / viewDir.z * (height * height_scale);
    return texCoords - p;
}

void main()
{
    // TexCoords
    vec3 viewDir = normalize(IN.TangentViewPos - IN.TangentFragPos);
    vec2 texCoords = IN.TexCoords;
//    vec2 texCoords = ParallaxMapping(IN.TexCoords, viewDir);

//    if(texCoords.x > 1.0 || texCoords.y > 1.0 || texCoords.x < 0.0 || texCoords.y < 0.0)
//        discard;

    // For normal mapping
    vec3 normal = texture(material.texture_normal1, texCoords).rgb;
    normal = normalize(normal * 2.0 - 1.0);

    vec4 color = texture(material.texture_diffuse1, texCoords);

    // ambient
    vec3 ambient  = light.ambient * color.rgb;

    // diffuse
    vec3 lightDir = normalize(IN.TangentLightPos - IN.TangentFragPos);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = light.diffuse * diff * color.rgb;

    // specular
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    //    vec3 specular = light.specular * spec * vec3(texture(material.texture_specular1, texCoords));
    vec3 specular = light.specular * spec;

    FragColor = vec4(ambient + diffuse + specular, color.a);
//    FragColor = texture(material.texture_specular1, IN.TexCoords);
}

