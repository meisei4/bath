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

uniform vec2 iResolution;

in vec2 fragTexCoord;
in vec2 uvNoiseOrigin;
in vec2 uvNoiseStep;

out vec4 finalColor;

const int DEPTH_CAP = 128;
const int STRIDE    = 16;
const int LOOPS     = DEPTH_CAP / STRIDE;

int marchDepthSIMD(vec2 origin, vec2 noise_step) {
    vec2  stepProj      = origin;
    vec2  stepProjDelta = noise_step * float(STRIDE);
    float threshold     = perlinSolidThreshold;
    float hitDepth      = float(DEPTH_CAP);
    float active_lane   = 1.0;
    float depthOffset   = 0.0;
    for (int i = 0; i < LOOPS; ++i) {
        float sample_coord = perlin_noise_iq(stepProj);
        float solid        = 1.0 - step(threshold, sample_coord);
        float laneMask     = solid * active_lane;
        hitDepth += (depthOffset - hitDepth) * laneMask;
        active_lane *= (1.0 - solid);
        stepProj += stepProjDelta;
        depthOffset += float(STRIDE);
    }
    return int(hitDepth);
}

void main() {
    int   depthSteps = marchDepthSIMD(uvNoiseOrigin, uvNoiseStep);
    float fSteps     = float(depthSteps);
    float waterMask  = step(0.5, fSteps);
    vec3  base       = mix(vec3(0.9), pow(waterColor, vec3(fSteps / waterDepthDivision)), waterMask);
    float darkenMask = step(float(waterStaticThreshold) + 0.5, fSteps);
    base             = mix(base, base * waterDarkenMultiplier, darkenMask);
    finalColor       = vec4(sqrt(base), 1.0);
}
