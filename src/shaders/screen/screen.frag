#version 330 core

uniform vec3 resolution;
uniform sampler2D texture_diffuse1;

in VS_OUTPUT {
    vec2 TexCoords;
} IN;

out vec4 FragColor;

void main()
{
    vec2 uv = gl_FragCoord.xy / resolution.xy;
    vec2 coord = (uv - 0.5) * vec2(resolution.x/resolution.y, 1.0) * 2.0;
    float rf = sqrt(dot(coord, coord)) * 0.25; // strength
    float rf2_1 = rf * rf + 1.0;
    float e = 1.0 / (rf2_1 * rf2_1); // smooth

    vec4 text = texture(texture_diffuse1, IN.TexCoords);
//    vec4 vignetteColor = vec4(0.07,0.07,0.15, 1.0 - e);

//    FragColor = vec4(mix(text.rgb, vignetteColor.rgb, vignetteColor.a), text.a);
//    FragColor = mix(text, vignetteColor, vignetteColor.a);
    FragColor = vec4(text.rgb * e, text.a);
}