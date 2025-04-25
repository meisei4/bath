// This shader is a fork/verbose rewrite of user Jakob Thomsen's 2025 MIT liscensed works found here:
// www.shadertoy.com/user/jt
// https://www.shadertoy.com/view/WXSGDt
// https://www.shadertoy.com/view/WX23Dc
// https://www.shadertoy.com/view/t3l3R7

// NOTE: There is some commented out shadertoy code throughout, if interested, see shadertoy port here:
// https://www.shadertoy.com/view/3XfSDX
shader_type canvas_item;
#include "res://Resources/Shaders/Glacier/noise_applied.gdshaderinc"
#include "res://Resources/Shaders/Glacier/color.gdshaderinc"
uniform vec2 iResolution;

// NOTE: Please only disable parallax if you want to just see the raw noise textures overlayed
// but keep in mind there is no gating to prevent the depth march,
#define ENABLE_PARALLAX
    #define PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR 6.0
    #define PARALLAX_NEAR_SCALAR                        0.025

#define ENABLE_MODE7
    #define HORIZON_LINE_ZERO_SCREEN_CENTER_UV_ADDITIVE 0.75
    #define MODE7_VERTICAL_ADDITIVE_SCALAR              0.25
    #define MODE7_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR    1.0
    #define MODE7_FAR_SCALAR                            0.005

#define CONSTANT_DEPTH_MARCH_STRIDE_LENGTH              1.0
#define ENABLE_VARIABLE_STRIDE_LENGTH_FOR_DEPTH_MARCH
    #define VARIABLE_STRIDE_LENGTH_INITIAL_VALUE        2.0
    #define VARIABLE_STRIDE_COMPRESSION_RATE            3.0

vec2 projectLayer(vec2 originalCoord, out float chosenNoiseScale) {
    #ifdef ENABLE_MODE7
        chosenNoiseScale = MODE7_FAR_SCALAR;
        originalCoord.y += MODE7_VERTICAL_ADDITIVE_SCALAR;
        originalCoord = originalCoord / (MODE7_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - originalCoord.y);
    #else
        chosenNoiseScale = PARALLAX_NEAR_SCALAR;
        originalCoord = originalCoord / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - originalCoord.y);
    #endif
    return originalCoord;
}

int depthMarch(vec2 normalizedCoord, float strideLength, float firstStepOffset){
    int depthCount = 0;
    while(true) {
        float firstStep = float(depthCount) + firstStepOffset;
        vec2 nextStep = normalizedCoord + vec2(0.0, firstStep / GLOBAL_COORD_SCALAR) * strideLength;
        float localNoiseScale;
        vec2 projectedCoord = projectLayer(nextStep, localNoiseScale);
        if(isSolidAtCoord(projectedCoord, localNoiseScale, TIME)) {
            break;
        }
        if(normalizedCoord.y + (float(depthCount) / GLOBAL_COORD_SCALAR) >= iResolution.y) {
            break;
        }
        depthCount++;
    }
    return depthCount;
}

vec2 projectTopLayerForParallax(vec2 normCoord, out float noiseScale) {
    #ifdef ENABLE_PARALLAX
        return projectLayer(normCoord, noiseScale);
    #else
        noiseScale = PARALLAX_NEAR_SCALAR;
        return normCoord;
    #endif
}

float getStrideLength(vec2 normCoord) {
    #ifdef ENABLE_VARIABLE_STRIDE_LENGTH_FOR_DEPTH_MARCH
        return (VARIABLE_STRIDE_LENGTH_INITIAL_VALUE - normCoord.y) / VARIABLE_STRIDE_COMPRESSION_RATE;
    #else
        return CONSTANT_DEPTH_MARCH_STRIDE_LENGTH;
    #endif
}

vec3 getTerrainColor(bool isSolid, int depth, vec2 normCoord) {
    if (isSolid) {
        return vec3(SOLID_REGION_BRIGHTNESS);
    } else {
        float e = float(depth) / WATER_DEPTH_DARKEN_DIVISOR;
        return pow(vec3(WATER_COLOR_R, WATER_COLOR_G, WATER_COLOR_B), vec3(e));
    }
}

vec3 applyWater(vec3 color, bool isSolid, int depth, vec2 normCoord) {
    #ifdef ENABLE_WATER
        return tintAndDarkenWater(color, isSolid, depth, normCoord, TIME);
    #else
        return color;
    #endif
}

//void mainImage(out vec4 fragColor, in vec2 fragCoord){
void fragment() {
    //vec2 normCoord = (fragCoord * 2.0 - iResolution.xy) / iResolution.y;
    vec2 normCoord = (FRAGCOORD.xy * 2.0 - iResolution.xy) / iResolution.y;
    float noiseScale;
    vec3 color;
    #ifdef ENABLE_MODE7
        if (normCoord.y >= HORIZON_LINE_ZERO_SCREEN_CENTER_UV_ADDITIVE){
            color = getSkyColorMode7(normCoord).rgb;
        } else //This funky if-else macro hack is because godot does not allow early returns in fragment shaders
    #endif
    {
        vec2 projectedTopLayerCoord = projectTopLayerForParallax(normCoord, noiseScale);
        float stride                = getStrideLength(normCoord);
        float firstStepOffset       = getFirstStepOffset(normCoord, TIME);
        int   depth                 = depthMarch(normCoord, stride, firstStepOffset);
        bool  isSolid               = isSolidAtCoord(projectedTopLayerCoord, noiseScale, TIME);
        vec3  terrain               = getTerrainColor(isSolid, depth, normCoord);
        vec3  terrainWithWater      = applyWater(terrain, isSolid, depth, normCoord);
        //color = terrainWithWater; //RAW give it a try... its gross... :^)
        color = sqrt(terrainWithWater); //GAMMAMAMA CORREECCTIONNN.... GOOD
    }
    COLOR = vec4(color.r, color.g, color.b, 1.0);
    //fragColor = vec4(color.r, color.g, color.b, 1.0);
}
