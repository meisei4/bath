#define ENABLE_WATER
    // pretty water color palette from: https://www.shadertoy.com/view/MlcGD7 by FabriceNeyret2
    #define WATER_COLOR_R                 0.1
    #define WATER_COLOR_G                 0.7
    #define WATER_COLOR_B                 0.8
    #define WATER_DARKEN_MULTIPLIER       0.5
    #define WATER_DEPTH_DARKEN_DIVISOR    9.0
    #define WATER_STATIC_DARKEN_THRESHOLD 12

//#define ENABLE_WATER_ANIMATION
    #define WATER_ANIM_DEPTH_MIN          8.0
    #define WATER_ANIM_DEPTH_MAX          30.0
    #define WATER_ANIM_COS_OFFSET         0.5
    #define WATER_ANIM_COS_AMPLITUDE      0.5
    #define WATER_ANIM_COS_FREQUENCY      0.2

//#define ENABLE_MODE7
    // same as water color palette/Rayleigh scattering for sky
    #define SKY_COLOR_R                   0.1
    #define SKY_COLOR_G                   0.7
    #define SKY_COLOR_B                   0.8
    #define SKY_GRADIENT_STRENGTH         15.0
    #define SOLID_REGION_BRIGHTNESS       0.9

vec4 getSkyColorMode7(vec2 normCoord) {
    vec4 baseSky = vec4(SKY_COLOR_R, SKY_COLOR_G, SKY_COLOR_B, 1.0);
    vec4 skyGradient = pow(baseSky, vec4(normCoord.y * SKY_GRADIENT_STRENGTH));
    return sqrt(skyGradient);
}

vec3 getTerrainColorWithWater(bool isSolid, int depth, vec2 normCoord) {
    if (isSolid) {
        return vec3(SOLID_REGION_BRIGHTNESS);
    } else {
        float e = float(depth) / WATER_DEPTH_DARKEN_DIVISOR;
        return pow(vec3(WATER_COLOR_R, WATER_COLOR_G, WATER_COLOR_B), vec3(e));
    }
}

uint getTerrainColorBinary(bool isSolid) {
    return isSolid ? 1u : 0u;
}
