//MISC MATH
#define M_PI 3.1415926
#define TAU (M_PI+M_PI)
#define PI_4 0.785398163397448309615660845819875721

#define ENABLE_WATER
    #define WATER_COLOR_R                      0.1
    #define WATER_COLOR_G                      0.7
    #define WATER_COLOR_B                      0.8
    #define WATER_DARKEN_MULTIPLIER            0.5
    #define WATER_DEPTH_DARKEN_DIVISOR         5.0
    #define WATER_STATIC_DARKEN_THRESHOLD      12

#define ENABLE_WATER_ANIMATION
    #define WATER_ANIM_DEPTH_MIN              8.0
    #define WATER_ANIM_DEPTH_MAX              20.0
    #define WATER_ANIM_COS_OFFSET             0.5
    #define WATER_ANIM_COS_AMPLITUDE          0.5
    #define WATER_ANIM_COS_FREQUENCY          0.1

#define SKY_COLOR_R                        0.1
#define SKY_COLOR_G                        0.7
#define SKY_COLOR_B                        0.8
#define SKY_GRADIENT_STRENGTH              15.0
#define SOLID_REGION_BRIGHTNESS            0.9

vec4 getSkyColorMode7(vec2 normCoord) {
    vec4 baseSky = vec4(SKY_COLOR_R, SKY_COLOR_G, SKY_COLOR_B, 1.0);
    vec4 skyGradient = pow(baseSky, vec4(normCoord.y * SKY_GRADIENT_STRENGTH));
    return sqrt(skyGradient);
}

vec3 tintAndDarkenWater(vec3 currentColor, bool isSolid, int depthCount, vec2 normCoord) {
    if(isSolid) return currentColor;
    float randomOffset = 0.0;
    #ifdef ENABLE_LAYER_RANDOMIZATION
        //randomOffset = hash12(normCoord * LAYER_OFFSET_MULTIPLIER + iTime);
        randomOffset = hash12(normCoord * LAYER_OFFSET_MULTIPLIER + TIME);
    #endif

    #ifdef ENABLE_WATER_ANIMATION
        float dynamicThreshold = mix(
            WATER_ANIM_DEPTH_MIN,
            WATER_ANIM_DEPTH_MAX,
            //WATER_ANIM_COS_OFFSET + WATER_ANIM_COS_AMPLITUDE * cos(tau * iTime * WATER_ANIM_COS_FREQUENCY)
            WATER_ANIM_COS_OFFSET + WATER_ANIM_COS_AMPLITUDE * cos(TAU * TIME * WATER_ANIM_COS_FREQUENCY)
        );
        if(depthCount > int(dynamicThreshold - randomOffset)) {
            currentColor *= WATER_DARKEN_MULTIPLIER;
        }
    #else
        if(depthCount > WATER_STATIC_DARKEN_THRESHOLD) {
            currentColor *= WATER_DARKEN_MULTIPLIER;
        }
    #endif
    return currentColor;
}