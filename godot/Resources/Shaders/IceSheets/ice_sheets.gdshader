// This shader is a fork/verbose rewrite of user Jakob Thomsen's 2025 MIT liscensed works found here:
// www.shadertoy.com/user/jt
// https://www.shadertoy.com/view/WXSGDt
// https://www.shadertoy.com/view/WX23Dc
// https://www.shadertoy.com/view/t3l3R7

// NOTE: There is some commented out shadertoy code throughout, if interested, see shadertoy port here:
// https://www.shadertoy.com/view/3XfSDX
shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/IceSheets/noise.gdshaderinc"
#include "res://Resources/Shaders/IceSheets/projections.gdshaderinc"
#include "res://Resources/Shaders/IceSheets/color.gdshaderinc"
uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest; // ← this will become our water background

uniform float iTime;

// NOTE: Please only disable parallax if you want to just see the raw noise textures overlayed
// but keep in mind there is no gating to prevent the depth march,

#define CONSTANT_DEPTH_MARCH_STRIDE_LENGTH              1.0
#define ENABLE_VARIABLE_STRIDE_LENGTH_FOR_DEPTH_MARCH
    #define VARIABLE_STRIDE_LENGTH_INITIAL_VALUE        2.0
    #define VARIABLE_STRIDE_COMPRESSION_RATE            3.0

int depthMarch(vec2 normalizedCoord, float strideLength, float firstStepOffset){
    int depthCount = 0;
    while(true) {
        float firstStep = float(depthCount) + firstStepOffset;
        vec2 nextStep = normalizedCoord + vec2(0.0, firstStep / GLOBAL_COORD_SCALAR) * strideLength;
        float localNoiseScale;
        vec2 projectedCoord = projectLayer(nextStep, localNoiseScale);
       //if(isSolidAtCoord(projectedCoord, localNoiseScale, TIME)) {
        if(isSolidAtCoord(projectedCoord, localNoiseScale, iTime)) {
            break;
        }
        if(normalizedCoord.y + (float(depthCount) / GLOBAL_COORD_SCALAR) >= iResolution.y) {
            break;
        }
        depthCount++;
    }
    return depthCount;
}

float getFirstStepOffset(vec2 normCoord) {
    #ifdef ENABLE_DEPTH_MARCH_FIRST_STEP_SCRAMBLING
        //return hash12(normCoord * FIRST_STEP_SCRAMBLER_SCALAR + TIME);
        return hash12(normCoord * FIRST_STEP_SCRAMBLER_SCALAR + iTime);
    #else
        return UNIFORM_DEPTH_MARCH_FIRST_STEP_SCALAR;
    #endif
}

float getStrideLength(vec2 normCoord) {
    #ifdef ENABLE_VARIABLE_STRIDE_LENGTH_FOR_DEPTH_MARCH
        return (VARIABLE_STRIDE_LENGTH_INITIAL_VALUE - normCoord.y) / VARIABLE_STRIDE_COMPRESSION_RATE;
    #else
        return CONSTANT_DEPTH_MARCH_STRIDE_LENGTH;
    #endif
}

vec3 tintAndDarkenWater(vec3 currentColor, bool isSolid, int depthCount, vec2 normCoord) {
    if(isSolid) return currentColor;
    float firstStepOffset = getFirstStepOffset(normCoord);
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


vec3 applyWater(vec3 color, bool isSolid, int depth, vec2 normCoord) {
    #ifdef ENABLE_WATER
        //return tintAndDarkenWater(color, isSolid, depth, normCoord);
        return tintAndDarkenWater(color, isSolid, depth, normCoord);
    #else
        return color;
    #endif
}

//void mainImage(out vec4 fragColor, in vec2 fragCoord){
void fragment() {
    //vec2 normCoord = (fragCoord * 2.0 - iResolution.xy) / iResolution.y;
    //vec2 normCoord = (FRAGCOORD.xy * 2.0 - iResolution.xy) / iResolution.y;
    vec2 fragCoord = UV * iResolution;
    // 2) now do exactly your same math
    vec2 normCoord = (fragCoord * 2.0 - iResolution) / iResolution.y;
    float noiseScale;
    vec3 color; //RGB
    float solidness_alpha_mask_for_collision; //A
    #ifdef ENABLE_MODE7
        if (normCoord.y >= HORIZON_LINE_ZERO_SCREEN_CENTER_UV_ADDITIVE){
            color = getSkyColorMode7(normCoord).rgb;
        } else //This funky if-else macro hack is because godot does not allow early returns in fragment shaders
    #endif
    {
        vec2 projectedTopLayerCoord = projectTopLayerForParallax(normCoord, noiseScale);
        float stride                = getStrideLength(normCoord);
        //float firstStepOffset       = getFirstStepOffset(normCoord, TIME);
        float firstStepOffset       = getFirstStepOffset(normCoord);
        int   depth                 = depthMarch(normCoord, stride, firstStepOffset);
        //bool  isSolid               = isSolidAtCoord(projectedTopLayerCoord, noiseScale, TIME);
        bool  isSolid               = isSolidAtCoord(projectedTopLayerCoord, noiseScale, iTime);
        vec3  terrain               = getTerrainColorWithWater(isSolid, depth, normCoord);
        vec3  terrainWithWater      = applyWater(terrain, isSolid, depth, normCoord);
        //color = terrainWithWater; //RAW give it a try... its gross... :^)
        color = sqrt(terrainWithWater); //GAMMAMAMA CORREECCTIONNN.... GOOD
        //TODO: this is pretty wacky, it draws the mask channel for the collisions
        // but we have to add a second pass to the MainImage that redraws the alpha as 1.0
        solidness_alpha_mask_for_collision = isSolid ? 1.0 : 0.0;
        //TODO: add the water projected texture somehow...
        //vec3 water = texture(iChannel0, UV * 0.5).rgb;
        //vec3 waterWithCaustics = color + water * 0.5;
        //color = isSolid
            //? color
            //: waterWithCaustics;
    }
    COLOR = vec4(color.r, color.g, color.b, solidness_alpha_mask_for_collision);
    //fragColor = vec4(color.r, color.g, color.b, 1.0);
}
