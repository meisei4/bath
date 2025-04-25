#include "res://Resources/Shaders/Glacier/noise_applied.gdshaderinc"


#define ENABLE_WATER
    // pretty water color palette from: https://www.shadertoy.com/view/MlcGD7 by FabriceNeyret2
    #define WATER_COLOR_R                 0.1
    #define WATER_COLOR_G                 0.7
    #define WATER_COLOR_B                 0.8
    #define WATER_DARKEN_MULTIPLIER       0.4
    #define WATER_DEPTH_DARKEN_DIVISOR    9.0
    #define WATER_STATIC_DARKEN_THRESHOLD 12

#define ENABLE_WATER_ANIMATION
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

float getFirstStepOffset(vec2 normCoord, float iTime) {
    #ifdef ENABLE_DEPTH_MARCH_FIRST_STEP_SCRAMBLING
        //return hash12(normCoord * FIRST_STEP_SCRAMBLER_SCALAR + TIME);
        return hash12(normCoord * FIRST_STEP_SCRAMBLER_SCALAR + iTime);
    #else
        return UNIFORM_DEPTH_MARCH_FIRST_STEP_SCALAR;
    #endif
}

vec3 tintAndDarkenWater(vec3 currentColor, bool isSolid, int depthCount, vec2 normCoord, float iTime) {
    if(isSolid) return currentColor;
    float firstStepOffset = getFirstStepOffset(normCoord, iTime);
    #ifdef ENABLE_WATER_ANIMATION
        float dynamicThreshold = mix(
            WATER_ANIM_DEPTH_MIN,
            WATER_ANIM_DEPTH_MAX,
            //WATER_ANIM_COS_OFFSET + WATER_ANIM_COS_AMPLITUDE * cos(TAU * iTime * WATER_ANIM_COS_FREQUENCY)
            WATER_ANIM_COS_OFFSET + WATER_ANIM_COS_AMPLITUDE * cos(TAU * TIME * WATER_ANIM_COS_FREQUENCY)
        );
        if(depthCount > int(dynamicThreshold - firstStepOffset)) {
            currentColor *= WATER_DARKEN_MULTIPLIER;
        }
    #else
        if(depthCount > WATER_STATIC_DARKEN_THRESHOLD) {
            currentColor *= WATER_DARKEN_MULTIPLIER;
        }
    #endif
    return currentColor;
}
