#version 330

uniform vec2  iResolution;
uniform float iTime;

#define M_PI 3.14159265358979323846
#define M_PI_4 0.785398163397448309616

// — Hash to vec2, then gradient
vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

vec2 grad(ivec2 z) { return hash22(vec2(z) * 123.456) * 2.0 - 1.0; }

// — Perlin noise (IQ style)
float perlin_noise_iq(vec2 p) {
    ivec2 i = ivec2(floor(p));
    vec2  f = fract(p);
    vec2  u = smoothstep(0.0, 1.0, f);

    float v00 = dot(grad(i + ivec2(0, 0)), f - vec2(0.0, 0.0));
    float v10 = dot(grad(i + ivec2(1, 0)), f - vec2(1.0, 0.0));
    float v01 = dot(grad(i + ivec2(0, 1)), f - vec2(0.0, 1.0));
    float v11 = dot(grad(i + ivec2(1, 1)), f - vec2(1.0, 1.0));

    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

#define NOISE_SCROLL_VELOCITY vec2(0.0, 0.1)
#define GLOBAL_COORD_SCALAR 180.0
#define STRETCH_SCALAR_Y 2.0
#define NOISE_COORD_OFFSET vec2(2.0, 0.0)
#define PERLIN_SOLID_THRESHOLD -0.03
#define ROT_ANGLE -M_PI_4
#define ROT mat2(vec2(cos(ROT_ANGLE), -sin(ROT_ANGLE)), vec2(sin(ROT_ANGLE), cos(ROT_ANGLE)))
#define UNIF_STRETCH_CORR sqrt(2.0)

float sampleNoise(vec2 coord, float localScale, float t) {
    vec2 s = (coord + t * NOISE_SCROLL_VELOCITY) * GLOBAL_COORD_SCALAR;
    s      = UNIF_STRETCH_CORR * vec2(s.x, s.y * STRETCH_SCALAR_Y);
    s      = ROT * s;
    return perlin_noise_iq(s * localScale - NOISE_COORD_OFFSET);
}

bool isSolidAtCoord(vec2 coord, float localScale, float t) {
    return sampleNoise(coord, localScale, t) < PERLIN_SOLID_THRESHOLD;
}

#define WATER_COLOR vec3(0.1, 0.7, 0.8)
#define WATER_DARKEN_MULT 0.5
#define WATER_DEPTH_DIV 9.0
#define WATER_STATIC_TH 12
#define SOLID_REGION_BRIGHTNESS 0.9

vec3 getTerrainColor(bool solid, int depth) {
    return solid ? vec3(SOLID_REGION_BRIGHTNESS) : pow(WATER_COLOR, vec3(float(depth) / WATER_DEPTH_DIV));
}

vec3 tintAndDarkenWater(vec3 col, bool solid, int depth) {
    if (solid)
        return col;
    return (depth > WATER_STATIC_TH) ? (col * WATER_DARKEN_MULT) : col;
}

#define PAR_DEPTH_D 6.0
#define PAR_NEAR_SCALE 0.025
#define STRIDE_LEN 1.0

vec2 projectLayer(vec2 c, out float localScale) {
    localScale = PAR_NEAR_SCALE;
    return c / (PAR_DEPTH_D - c.y);
}

int depthMarch(vec2 nc, float t) {
    int d = 0;
    while (true) {
        vec2  stepPos = nc + vec2(0.0, float(d) / GLOBAL_COORD_SCALAR) * STRIDE_LEN;
        float ls;
        vec2  pc = projectLayer(stepPos, ls);

        if (isSolidAtCoord(pc, ls, t))
            break;
        if (nc.y + float(d) / GLOBAL_COORD_SCALAR >= iResolution.y)
            break;

        d++;
    }
    return d;
}

in vec2  fragTexCoord;
out vec4 finalColor;

void main() {
    vec2  fc = fragTexCoord * iResolution;
    vec2  nc = (fc * 2.0 - iResolution) / iResolution.y;
    float ls;
    vec2  topPC   = projectLayer(nc, ls);
    int   depth   = depthMarch(nc, iTime);
    bool  solid   = isSolidAtCoord(topPC, ls, iTime);
    vec3  terrain = getTerrainColor(solid, depth);
    vec3  water   = tintAndDarkenWater(terrain, solid, depth);
    vec3  col     = sqrt(water);
    finalColor    = vec4(col, 1.0);
}
