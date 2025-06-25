#version 330

vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

vec2 grad(ivec2 z) { return hash22(vec2(z) * 123.456) * 2.0 - 1.0; }

float perlin_noise_iq(vec2 p) {
    ivec2 i   = ivec2(floor(p));
    vec2  f   = fract(p);
    vec2  u   = smoothstep(0.0, 1.0, f);
    float v00 = dot(grad(i + ivec2(0, 0)), f - vec2(0.0, 0.0));
    float v10 = dot(grad(i + ivec2(1, 0)), f - vec2(1.0, 0.0));
    float v01 = dot(grad(i + ivec2(0, 1)), f - vec2(0.0, 1.0));
    float v11 = dot(grad(i + ivec2(1, 1)), f - vec2(1.0, 1.0));
    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

const float perlinSolidThreshold  = -0.03;
const vec3  waterColor            = vec3(0.1, 0.7, 0.8);
const float waterDarkenMultiplier = 0.5;
const float waterDepthDivision    = 9.0;
const int   waterStaticThreshold  = 12;

uniform vec2  iResolution;
uniform float iTime;

in vec2 fragTexCoord;
in vec2 vertexNormCoord;
in vec2 vertexTopLayerProjection;
in vec2 vertexNoiseOrigin;
in vec2 vertexNoiseStep;

out vec4 finalColor;

int marchDepthInProjectedNoiseSpace() {
    vec2 sampleProj = vertexNoiseOrigin;
    for (int depthStep = 0;; depthStep++) {
        if (perlin_noise_iq(sampleProj) < perlinSolidThreshold) {
            return depthStep;
        }
        sampleProj += vertexNoiseStep;
    }
}

void main() {
    vec2 fragCoord            = fragTexCoord * iResolution;
    vec2 normalizedCoordinate = vertexNormCoord;
    vec2 topProjection        = vertexTopLayerProjection;
    int  depthSteps           = marchDepthInProjectedNoiseSpace();
    bool isSolid              = perlin_noise_iq(vertexNoiseOrigin) < perlinSolidThreshold ? true : false;
    vec3 baseColor            = isSolid ? vec3(0.9) : pow(waterColor, vec3(float(depthSteps) / waterDepthDivision));
    if (!isSolid && depthSteps > waterStaticThreshold) {
        baseColor *= waterDarkenMultiplier;
    }
    finalColor = vec4(sqrt(baseColor), 1.0);
}
