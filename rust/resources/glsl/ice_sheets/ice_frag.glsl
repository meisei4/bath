#version 330

vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

vec2 grad(ivec2 z) { return hash22(vec2(z) * 123.456) * 2.0 - 1.0; }

float perlin_noise_iq(vec2 p) {
    ivec2 i = ivec2(floor(p));
    vec2 f = fract(p);
    vec2 u = smoothstep(0.0, 1.0, f);
    float v00 = dot(grad(i + ivec2(0, 0)), f - vec2(0.0));
    float v10 = dot(grad(i + ivec2(1, 0)), f - vec2(1.0, 0.0));
    float v01 = dot(grad(i + ivec2(0, 1)), f - vec2(0.0, 1.0));
    float v11 = dot(grad(i + ivec2(1, 1)), f - vec2(1.0, 1.0));
    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

#define M_PI 3.14159265358979323846
#define M_PI_4 0.785398163397448309616

#define NOISE_SCROLL_VELOCITY vec2(0.0, 0.1)
#define GLOBAL_COORD_SCALAR 180.0
#define STRETCH_SCALAR_Y 2.0
#define NOISE_COORD_OFFSET vec2(2.0, 0.0)
#define PERLIN_SOLID_THRESHOLD -0.03
#define ROT_ANGLE - M_PI_4
#define ROT mat2(vec2(cos(ROT_ANGLE), - sin(ROT_ANGLE)), vec2(sin(ROT_ANGLE), cos(ROT_ANGLE)))
#define UNIF_STRETCH_CORR sqrt(2.0)

#define WATER_COLOR vec3(0.1, 0.7, 0.8)
#define WATER_DARKEN_MULT 0.5
#define WATER_DEPTH_DIV 9.0
#define WATER_STATIC_TH 12
#define SOLID_REGION_BRIGHTNESS 0.9

#define PAR_DEPTH_D 6.0
#define PAR_NEAR_SCALE 0.025
#define STRIDE_LEN 1.0

float sampleNoise(vec2 coord, float localScale, float t) {
    vec2 s = (coord + t * NOISE_SCROLL_VELOCITY) * GLOBAL_COORD_SCALAR;
    s = UNIF_STRETCH_CORR * vec2(s.x, s.y * STRETCH_SCALAR_Y);
    s = ROT * s;
    return perlin_noise_iq(s * localScale - NOISE_COORD_OFFSET);
}

bool isSolidAtCoord(vec2 coord, float localScale, float t) {
    return sampleNoise(coord, localScale, t) < PERLIN_SOLID_THRESHOLD;
}

int depthMarch(vec2 normCoord, float localScale, float t, vec2 resolution) {
    float y_px = 0.5 * (normCoord.y + 1.0) * resolution.y;
    int   d    = 0;                    /* depth in pixel rows */
    while (true) {
        if (y_px >= resolution.y)
            break;
        float nY         = (y_px / resolution.y) * 2.0 - 1.0;
        vec2  stepPosNDC = vec2(normCoord.x, nY);
        vec2  pc         = stepPosNDC / (PAR_DEPTH_D - stepPosNDC.y);
        if (isSolidAtCoord(pc, localScale, t))
            break;
        y_px += STRIDE_LEN;
        d    += 1;
    }
    return d;
}

uniform float iTime;
uniform vec2  iResolution;

in  vec2 fragTexCoord;
out vec4 finalColor;

void main() {
    vec2 fragCoord = fragTexCoord * iResolution;
    vec2 normCoord = (fragCoord * 2.0 - iResolution) / iResolution.y;
    float localScale    = PAR_NEAR_SCALE;
    vec2  topPC         = normCoord / (PAR_DEPTH_D - normCoord.y);
    bool rawSolid = isSolidAtCoord(topPC, localScale, iTime);
    int depth = depthMarch(normCoord, localScale, iTime, iResolution);
    bool solid;
    if (depth > 0 && rawSolid)
        solid = true;
    else
        solid = false;

    vec3 terrain = solid
    ? vec3(SOLID_REGION_BRIGHTNESS)
    : pow(WATER_COLOR, vec3(float(depth) / WATER_DEPTH_DIV));

    vec3 water = solid
    ? terrain
    : (depth > WATER_STATIC_TH
    ? terrain * WATER_DARKEN_MULT
    : terrain);

    vec3 col = sqrt(water);
    finalColor = vec4(col, 1.0);
}
