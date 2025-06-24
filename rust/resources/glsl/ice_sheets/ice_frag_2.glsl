#version 330

uniform float iTime;

uniform vec2 uNoiseScrollVel;
uniform float uGlobalCoordScalar;
uniform float uStretchY;
uniform vec2 uNoiseCoordOffset;
uniform float uUnifStretchCorr;
uniform float uRotCos;
uniform float uRotSin;
uniform float uPerlinSolidTh;

uniform float uWaterColR;
uniform float uWaterColG;
uniform float uWaterColB;
uniform float uWaterDarkenMult;
uniform float uWaterDepthDiv;
uniform float uWaterStaticTh;

uniform float uSolidBrightness;
uniform float uParDepth;
uniform float uParNearScale;
uniform float uStrideLen;

in vec2 v_normCoord;
in vec2 v_noiseCoord;

out vec4 finalColor;

#define M_PI 3.14159265358979323846
#define M_PI_4 0.785398163397448309616

vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}
vec2 grad(ivec2 z) { return hash22(vec2(z) * 123.456) * 2.0 - 1.0; }

float perlin_noise_iq(vec2 p) {
    ivec2 i = ivec2(floor(p));
    vec2 f = fract(p), u = smoothstep(0.0, 1.0, f);
    float v00 = dot(grad(i + ivec2(0, 0)), f - vec2(0.0));
    float v10 = dot(grad(i + ivec2(1, 0)), f - vec2(1.0, 0.0));
    float v01 = dot(grad(i + ivec2(0, 1)), f - vec2(0.0, 1.0));
    float v11 = dot(grad(i + ivec2(1, 1)), f - vec2(1.0, 1.0));
    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

float sampleNoise(vec2 c, float s, float t) {
    vec2 q = (c + t * uNoiseScrollVel) * uGlobalCoordScalar;
    q = uUnifStretchCorr * vec2(q.x, q.y * uStretchY);
    mat2 rot = mat2(vec2(uRotCos, -uRotSin), vec2(uRotSin, uRotCos));
    q = rot * q;
    return perlin_noise_iq(q * s - uNoiseCoordOffset);
}

bool isSolidAtCoord(vec2 c, float ls, float t) { return sampleNoise(c, ls, t) < uPerlinSolidTh; }

vec3 getTerrainColor(bool solid, int d) {
    if (solid)
    return vec3(uSolidBrightness);
    float e = float(d) / uWaterDepthDiv;
    return pow(vec3(uWaterColR, uWaterColG, uWaterColB), vec3(e));
}

vec3 tintAndDarkenWater(vec3 col, bool solid, int d) {
    if (solid)
    return col;
    return (d > int(uWaterStaticTh)) ? col * uWaterDarkenMult : col;
}

vec2 projectLayer(vec2 c, out float ls) {
    ls = uParNearScale;
    return c / (uParDepth - c.y);
}

int depthMarch(vec2 nc, float t) {
    int d = 0;
    while (true) {
        vec2 stepPos = nc + vec2(0.0, float(d) / uGlobalCoordScalar) * uStrideLen;
        float ls;
        vec2 pc = projectLayer(stepPos, ls);
        if (isSolidAtCoord(pc, ls, t))
        break;
        if (nc.y + float(d) / uGlobalCoordScalar >= 1.0)
        break;
        d++;
    }
    return d;
}

void main() {
    float ls = uParNearScale;
    vec2 topPC = v_noiseCoord;

    int depth = depthMarch(v_normCoord, iTime);

    bool solid = isSolidAtCoord(topPC, ls, iTime);
    vec3 terr = getTerrainColor(solid, depth);
    vec3 color = tintAndDarkenWater(terr, solid, depth);

    finalColor = vec4(sqrt(color), 1.0);
}
