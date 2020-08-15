#version 330 core

uniform vec3 resolution;
uniform sampler2D texture_diffuse1;

in VS_OUTPUT {
    vec2 TexCoords;
} IN;

out vec4 FragColor;

void main()
{
    /**
        Vinette
    **/
    vec2 uv = gl_FragCoord.xy / resolution.xy;
    vec2 coord = (uv - 0.5) * vec2(resolution.x/resolution.y, 1.0) * 2.0;
    float rf = sqrt(dot(coord, coord)) * 0.25; // strength
    float rf2_1 = rf * rf + 1.0;
    float e = 1.0 / (rf2_1 * rf2_1); // smooth

    FragColor = texture(texture_diffuse1, IN.TexCoords);

    /**
        FXAA
    **/
    // https://github.com/McNopper/OpenGL/blob/master/Example42/shader/fxaa.frag.glsl

    //hardcoded data
    vec2 u_texelStep = vec2(1.0/1024.0, 1.0/600.0);
    int u_showEdges = 0;

    float u_lumaThreshold = 0.45;
    float u_mulReduce = 1.0 / 20.0;
    float u_minReduce = 1.0 / 128.0;
    float u_maxSpan = 8.0;

    //code
    vec3 rgbM = texture(texture_diffuse1, IN.TexCoords).rgb;

    vec3 rgbNW = textureOffset(texture_diffuse1, IN.TexCoords, ivec2(-1, 1)).rgb;
    vec3 rgbNE = textureOffset(texture_diffuse1, IN.TexCoords, ivec2(1, 1)).rgb;
    vec3 rgbSW = textureOffset(texture_diffuse1, IN.TexCoords, ivec2(-1, -1)).rgb;
    vec3 rgbSE = textureOffset(texture_diffuse1, IN.TexCoords, ivec2(1, -1)).rgb;

    const vec3 toLuma = vec3(0.299, 0.587, 0.114);

    float lumaNW = dot(rgbNW, toLuma);
    float lumaNE = dot(rgbNE, toLuma);
    float lumaSW = dot(rgbSW, toLuma);
    float lumaSE = dot(rgbSE, toLuma);
    float lumaM = dot(rgbM, toLuma);

    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

    if (lumaMax - lumaMin <= lumaMax * u_lumaThreshold)
    {
        // ... do no AA and return.
        FragColor = vec4(rgbM, 1.0);

        // Output
        FragColor = vec4(FragColor.rgb * e, FragColor.a);

        return;
    }

    vec2 samplingDirection;
    samplingDirection.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    samplingDirection.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));

    float samplingDirectionReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * 0.25 * u_mulReduce, u_minReduce);
    float minSamplingDirectionFactor = 1.0 / (min(abs(samplingDirection.x), abs(samplingDirection.y)) + samplingDirectionReduce);

    samplingDirection = clamp(samplingDirection * minSamplingDirectionFactor, vec2(-u_maxSpan), vec2(u_maxSpan)) * u_texelStep;

    // Inner samples on the tab.
    vec3 rgbSampleNeg = texture(texture_diffuse1, IN.TexCoords + samplingDirection * (1.0/3.0 - 0.5)).rgb;
    vec3 rgbSamplePos = texture(texture_diffuse1, IN.TexCoords + samplingDirection * (2.0/3.0 - 0.5)).rgb;

    vec3 rgbTwoTab = (rgbSamplePos + rgbSampleNeg) * 0.5;

    // Outer samples on the tab.
    vec3 rgbSampleNegOuter = texture(texture_diffuse1, IN.TexCoords + samplingDirection * (0.0/3.0 - 0.5)).rgb;
    vec3 rgbSamplePosOuter = texture(texture_diffuse1, IN.TexCoords + samplingDirection * (3.0/3.0 - 0.5)).rgb;

    vec3 rgbFourTab = (rgbSamplePosOuter + rgbSampleNegOuter) * 0.25 + rgbTwoTab * 0.5;

    // Calculate luma for checking against the minimum and maximum value.
    float lumaFourTab = dot(rgbFourTab, toLuma);

    // Are outer samples of the tab beyond the edge ...
    if (lumaFourTab < lumaMin || lumaFourTab > lumaMax)
    {
        // ... yes, so use only two samples.
        FragColor = vec4(rgbTwoTab, 1.0);
    }
    else
    {
        // ... no, so use four samples.
        FragColor = vec4(rgbFourTab, 1.0);
    }

    // Show edges for debug purposes.
    if (u_showEdges != 0)
    {
        FragColor.r = 1.0;
    }

    /**
        OUTPUT
    **/
    FragColor = vec4(FragColor.rgb * e, FragColor.a);
}