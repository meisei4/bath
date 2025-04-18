shader_type canvas_item;

#include "res://Resources/Shaders/Glacier/common.gdshaderinc"

uniform vec2 iResolution;
//This shader is a fork and study of user www.shadertoy.com/user/jt 's works found here:
//https://www.shadertoy.com/view/WXSGDt,
//https://www.shadertoy.com/view/WX23Dc,
//https://www.shadertoy.com/view/t3l3R7

#define NOISE_SCROLL_VELOCITY              vec2(0.0, 0.1)
#define GLOBAL_COORD_SCALAR                 180.0

#define ENABLE_STRETCH_NOISE_TEXTURE
    #define STRETCH_SCALAR_X                1.0
    #define STRETCH_SCALAR_Y                2.0

#define ENABLE_ROTATION
    #define ROTATION_ANGLE        -PI_4
    //#define ROTATION_ANGLE        (-PI_4 * 0.5)
    #define ROTATION_MATRIX       mat2(vec2(cos(ROTATION_ANGLE), -sin(ROTATION_ANGLE)),\
                                        vec2(sin(ROTATION_ANGLE),  cos(ROTATION_ANGLE)))


#define ENABLE_PRE_ROTATION_STRETCH_CORRECTION
    #define UNIFORM_CORRECTION_SCALAR sqrt(2.0)
    //#define UNIFORM_CORRECTION_SCALAR (sqrt((pow(STRETCH_SCALAR_X, 2.0) + pow(STRETCH_SCALAR_Y, 2.0)) / 2.0))

//#define ENABLE_COMPOSED_AFFINE_TRANSFORMATIONS
    #define UNIFORM_SCALE_ROTATION_COMPOSED mat2(vec2(1.0,  1.0),  \
                                                  vec2(-1.0,  1.0))

//#define USE_VORONOI
    #define VORONOI_SOLID_THRESHOLD             0.34

#define PERLIN_SOLID_THRESHOLD             -0.03
#define NOISE_COORD_OFFSET                 vec2(2.0, 0.0)

float sampleNoise(vec2 coord, float localNoiseScale) {
    //float x_displacement = iTime * NOISE_SCROLL_VELOCITY.x;
    //float y_displacement = iTime * NOISE_SCROLL_VELOCITY.y;
    float x_displacement = TIME * NOISE_SCROLL_VELOCITY.x;
    float y_displacement = TIME * NOISE_SCROLL_VELOCITY.y;
    vec2 displaced_coordinate = vec2(
        coord.x + x_displacement,
        coord.y + y_displacement
    );
    vec2 scaled_coordinate = displaced_coordinate * GLOBAL_COORD_SCALAR;
    vec2 stretched_coordinate = vec2(
        scaled_coordinate.x * STRETCH_SCALAR_X,
        scaled_coordinate.y * STRETCH_SCALAR_Y
    );

    #ifdef ENABLE_COMPOSED_AFFINE_TRANSFORMATIONS
        vec2 uniform_scaled_rotated_coordinate = UNIFORM_SCALE_ROTATION_COMPOSED * stretched_coordinate;
        vec2 local_noise_scaled_coordinate = uniform_scaled_rotated_coordinate * localNoiseScale;
    #else
        vec2 uniform_scaled_coordinate = UNIFORM_CORRECTION_SCALAR * stretched_coordinate;
        vec2 rotated_coordinate = ROTATION_MATRIX * uniform_scaled_coordinate;
        vec2 local_noise_scaled_coordinate = rotated_coordinate * localNoiseScale;
    #endif

    vec2 final_noise_coordinate = local_noise_scaled_coordinate - NOISE_COORD_OFFSET;
    #ifdef USE_VORONOI
        float sampled_noise = voronoi_edge_fast(final_noise_coordinate);
    #else
        float sampled_noise = perlin_noise_iq(final_noise_coordinate);
    #endif
    return sampled_noise;
}


bool isSolidAtCoord(vec2 coord, float localNoiseScale) {
    float noiseValue = sampleNoise(coord, localNoiseScale);
    #ifdef USE_VORONOI
        return (noiseValue > VORONOI_SOLID_THRESHOLD);
    #else
        return (noiseValue < PERLIN_SOLID_THRESHOLD);
    #endif
}

#define ENABLE_PARALLAX
    #define NEAR_LAYER_DIVISOR_OFFSET          6.0
    #define NOISE_SCALE_NEAR                   0.025

#define ENABLE_MODE7
    #define HORIZON_LINE                       0.75
    #define DISTANT_LAYER_VERTICAL_OFFSET      0.25
    #define DISTANT_LAYER_DIVISOR_OFFSET       1.0
    #define NOISE_SCALE_DISTANT                0.005

#define ENABLE_PERSPECTIVE
    #define PERSPECTIVE_OFFSET                 2.0
    #define PERSPECTIVE_DIVISOR                3.0
    #define PERSPECTIVE_SCALE_NEUTRAL          1.0

#define ENABLE_LAYER_RANDOMIZATION
    #define LAYER_OFFSET_MULTIPLIER            123.456
    #define LAYER_OFFSET_DEFAULT               0.0

vec2 applyLayerPerspective(vec2 originalCoord, out float chosenNoiseScale) {
    #ifdef ENABLE_MODE7
        chosenNoiseScale = NOISE_SCALE_DISTANT;
        originalCoord.y += DISTANT_LAYER_VERTICAL_OFFSET;
        originalCoord = originalCoord / (DISTANT_LAYER_DIVISOR_OFFSET - originalCoord.y);
    #else
        chosenNoiseScale = NOISE_SCALE_NEAR;
        originalCoord = originalCoord / (NEAR_LAYER_DIVISOR_OFFSET - originalCoord.y);
    #endif
    return originalCoord;
}

int countFluidLayers(vec2 normalizedCoord, float perspectiveFactor, float layerOffset){
    int depthCount = 0;
    while(true) {
        float depthAsFloat = float(depthCount) + layerOffset;
        vec2 testCoord = normalizedCoord + vec2(0.0, depthAsFloat / GLOBAL_COORD_SCALAR) * perspectiveFactor;
        float localNoiseScale;
        vec2 finalCoord = applyLayerPerspective(testCoord, localNoiseScale);
        if(isSolidAtCoord(finalCoord, localNoiseScale)) {
            break;
        }
        if(normalizedCoord.y + (float(depthCount) / GLOBAL_COORD_SCALAR) >= iResolution.y) {
            break;
        }
        depthCount++;
    }
    return depthCount;
}

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
            WATER_ANIM_COS_OFFSET + WATER_ANIM_COS_AMPLITUDE * cos(tau * TIME * WATER_ANIM_COS_FREQUENCY)
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

//void mainImage(out vec4 fragColor, in vec2 fragCoord){
void fragment(){
    //vec2 normCoord = (fragCoord * 2.0 - iResolution.xy) / iResolution.y;
    vec2 normCoord = (FRAGCOORD.xy * 2.0 - iResolution.xy) / iResolution.y;
    //TODO: rewrite for godot has to allow for a break/jump somehow
    //#ifdef ENABLE_MODE7
        //if(normCoord.y >= HORIZON_LINE) {
            ////fragColor = getSkyColorMode7(normCoord);
            //COLOR = getSkyColorMode7(normCoord);
        //}
    //#endif

    vec2 transformCoord = normCoord;
    float localNoiseScale = NOISE_SCALE_NEAR;
    #ifdef ENABLE_PARALLAX
        transformCoord = applyLayerPerspective(normCoord, localNoiseScale);
    #endif

    float perspectiveFactor = PERSPECTIVE_SCALE_NEUTRAL;
    #ifdef ENABLE_PERSPECTIVE
        perspectiveFactor = (PERSPECTIVE_OFFSET - normCoord.y) / PERSPECTIVE_DIVISOR;
    #endif

    float layerOffsetVal = LAYER_OFFSET_DEFAULT;
    #ifdef ENABLE_LAYER_RANDOMIZATION
        //layerOffsetVal = hash12(normCoord * LAYER_OFFSET_MULTIPLIER + iTime);
        layerOffsetVal = hash12(normCoord * LAYER_OFFSET_MULTIPLIER + TIME);
    #endif

    int fluidDepth = countFluidLayers(normCoord, perspectiveFactor, layerOffsetVal);
    bool fragmentIsSolid = isSolidAtCoord(transformCoord, localNoiseScale);
    vec3 color;
    if(fragmentIsSolid) {
        color = vec3(SOLID_REGION_BRIGHTNESS, SOLID_REGION_BRIGHTNESS, SOLID_REGION_BRIGHTNESS); //GRAYSCALE??????
    } else {
        float exponent = float(fluidDepth) / WATER_DEPTH_DARKEN_DIVISOR;
        vec3 water = vec3(WATER_COLOR_R, WATER_COLOR_G, WATER_COLOR_B);
        color = pow(water, vec3(exponent, exponent, exponent));
    }

    #ifdef ENABLE_WATER
        color = tintAndDarkenWater(color, fragmentIsSolid, fluidDepth, normCoord);
    #endif

    color = sqrt(color);
    //fragColor = vec4(color.r, color.g, color.b, 1.0);
    COLOR = vec4(color.r, color.g, color.b, 1.0);
}
