#define ENABLE_PARALLAX
    #define PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR 6.0
    #define PARALLAX_NEAR_SCALAR                        0.025

//#define ENABLE_MODE7
    #define HORIZON_LINE_ZERO_SCREEN_CENTER_UV_ADDITIVE 0.75
    #define MODE7_VERTICAL_ADDITIVE_SCALAR              0.25
    #define MODE7_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR    1.0
    #define MODE7_FAR_SCALAR                            0.005

vec2 projectLayer1(vec2 originalCoord, out float chosenNoiseScale) {
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
float affineConstsA(float d) {
    return 0.5 * log((d + 1.0) / (d - 1.0));
}

float affineConstsB(float d) {
    return 1.5 * (d * log((d + 1.0) / (d - 1.0)) - 2.0);
}

vec2 projectLayer(vec2 orig, out float chosenNoiseScale) {
    chosenNoiseScale = PARALLAX_NEAR_SCALAR;
    float a      = affineConstsA(PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR);
    float b      = affineConstsB(PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR);
    float kX     = a;
    float scaleY = a + b * orig.y;
    return vec2(orig.x * kX, orig.y * scaleY);
}


vec2 projectTopLayerForParallax(vec2 normCoord, out float noiseScale) {
    #ifdef ENABLE_PARALLAX
        return projectLayer(normCoord, noiseScale);
    #else
        noiseScale = PARALLAX_NEAR_SCALAR;
        return normCoord;
    #endif
}