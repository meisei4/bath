shader_type canvas_item;


#include "res://Resources/shaders/water/constants.gdshaderinc"
#include "res://Resources/shaders/ice_sheets/projections.gdshaderinc"


uniform vec2 iResolution;
uniform sampler2D iChannel0: repeat_enable;
uniform sampler2D iChannel1: repeat_enable;
uniform sampler2D iChannel2: repeat_enable;
uniform sampler2D iChannel3;

//#define PIXELATE_UV
#define SHOW_NOISE_DISP_MAP_1
#define SHOW_NOISE_DISP_MAP_2
#define SHOW_CAUSTICS_DISP_MAP_1
#define SHOW_CAUSTICS_DISP_MAP_2
#define SHOW_BACKGROUND
//#define RIPPLE_EFFECT

#define PARALLAX_ZOOM  4.0

vec2 normalizeCoord(vec2 uv) {
    float aspect = iResolution.x / iResolution.y;
    return vec2(
        (uv.x * 2.0 - 1.0) * aspect,
        (uv.y * 2.0 - 1.0)
    );
}
vec2 denormalizeCoord(vec2 norm) {
    return norm * 0.5 + 0.5;
}

//TODO: figure out how to fix the wrapping to not destroy the projection
#define SCROLL_PHASE_DURATION 8.0

vec2 scrollInNormSpace(vec2 norm, vec2 velUV, vec2 offUV, float noiseScale) {
    float aspect = iResolution.x / iResolution.y;
    vec2 velNorm = vec2(
        velUV.x * 2.0 * aspect * PARALLAX_ZOOM,
        velUV.y * 2.0 * PARALLAX_ZOOM
    );
    vec2 offNorm = vec2(
        offUV.x * 2.0 * aspect,
        offUV.y * 2.0
    );
    float cycle = SCROLL_PHASE_DURATION * 2.0;
    float t     = mod(TIME, cycle);
    float wave  = (t < SCROLL_PHASE_DURATION)
                  ? t
                  : (cycle - t);
    vec2 disp = velNorm * wave;
    return norm + offNorm + disp;
}


vec4 sample_disp_map_projected(
    sampler2D tex,
    vec2     norm,
    float    noiseScale,
    vec2     velUV,
    vec2     offUV,
    float    intensity
) {
    norm   *= PARALLAX_ZOOM;
    vec2   vel = velUV * PARALLAX_ZOOM;
    vec2   off = offUV * PARALLAX_ZOOM;
    vec2 sc = scrollInNormSpace(norm, vel, off, noiseScale);
    float dummy;
    vec2 pr = projectTopLayerForParallax(sc, dummy);
    vec2 uv = denormalizeCoord(pr);
    float v = texture(tex, uv).r;
    return vec4(v * intensity);
}

vec4 sample_background_with_disp_map_projected(
    sampler2D tex,
    vec2     norm,
    float    noiseScale,
    vec4     disp,
    float    warp
) {
    norm *= PARALLAX_ZOOM;
    float aspect = iResolution.x / iResolution.y;
    vec2  w      = vec2(disp.r * warp * 2.0 * aspect * PARALLAX_ZOOM,
                       disp.r * warp * 2.0 * PARALLAX_ZOOM);
    vec2  sc     = norm + w;
    float dummy;
    vec2 pr = projectTopLayerForParallax(sc, dummy);
    vec2 uv = denormalizeCoord(pr);
    return texture(tex, uv);
}

vec2 compute_ripple_offset(vec2 uv) {
    float hc = texture(iChannel3, uv).r;
    float hl = texture(iChannel3, uv - vec2(NORMAL_SAMPLE_OFFSET,0)).r;
    float hr = texture(iChannel3, uv + vec2(NORMAL_SAMPLE_OFFSET,0)).r;
    float hu = texture(iChannel3, uv + vec2(0, NORMAL_SAMPLE_OFFSET)).r;
    float hd = texture(iChannel3, uv - vec2(0, NORMAL_SAMPLE_OFFSET)).r;
    vec3 n = normalize(vec3(hr-hl, hu-hd, NORMAL_Z_SCALE * NORMAL_SAMPLE_OFFSET));
    vec3 r = refract(INCIDENT_DIRECTION, n, REFRACTION_INDEX_RATIO);
    return r.xy;
}

float compute_effective_opacity(
    vec4 n1, vec4 n2,
    vec4 c1, vec4 c2
) {
    float sumN   = n1.r + n2.r;
    float sumAll = sumN + c1.r + c2.r;
    float a = BLURRY_ALPHA;
    if (sumN   > NOISE_DISP_INDUCED_INTENSITY_THRESHOLD) a = NORMAL_ALPHA;
    if (sumAll > ALL_DISP_MAP_INDUCED_INTENSITY_THRESHOLD) a = FULL_ALPHA;
    return a;
}

void fragment() {
    vec2 uv0 = FRAGCOORD.xy / iResolution;

    #ifdef PIXELATE_UV
        uv0 = pixelate_uv(uv0);
    #endif
    #ifdef RIPPLE_EFFECT
        uv0 += compute_ripple_offset(uv0);
    #endif

    vec2 norm = normalizeCoord(uv0);
    float noiseScale;
    vec2 pr = projectTopLayerForParallax(norm, noiseScale);
    vec2 uv = denormalizeCoord(pr);

    vec4 noise1 = vec4(0), noise2 = vec4(0);
    vec4 caust1 = vec4(0), caust2 = vec4(0);
    vec4 bg     = vec4(0);

    #ifdef SHOW_NOISE_DISP_MAP_1
        noise1 = sample_disp_map_projected(
            iChannel0, norm, noiseScale,
            NOISE_DISP_MAP_1_SCROLL_VELOCITY,
            NOISE_DISP_MAP_1_INITIAL_OFFSET,
            NOISE_DISP_MAP_DIMMING_FACTOR
        );
    #endif
    #ifdef SHOW_NOISE_DISP_MAP_2
        noise2 = sample_disp_map_projected(
            iChannel0, norm, noiseScale,
            NOISE_DISP_MAP_2_SCROLL_VELOCITY,
            NOISE_DISP_MAP_2_INITIAL_OFFSET,
            NOISE_DISP_MAP_DIMMING_FACTOR
        );
    #endif
    #ifdef SHOW_CAUSTICS_DISP_MAP_1
        caust1 = sample_disp_map_projected(
            iChannel2, norm, noiseScale,
            CAUSTICS_DISP_MAP_1_SCROLL_VELOCITY,
            ZERO_POSITIONAL_OFFSET,
            CAUSTICS_DISP_MAP_DIMMING_FACTOR
        );
    #endif
    #ifdef SHOW_CAUSTICS_DISP_MAP_2
        caust2 = sample_disp_map_projected(
            iChannel2, norm, noiseScale,
            CAUSTICS_DISP_MAP_2_SCROLL_VELOCITY,
            ZERO_POSITIONAL_OFFSET,
            CAUSTICS_DISP_MAP_DIMMING_FACTOR
        );
    #endif
    #ifdef SHOW_BACKGROUND
        bg = sample_background_with_disp_map_projected(
            iChannel1, norm, noiseScale,
            noise1,
            BACKGROUND_DISP_WARP_FACTOR
        );
    #endif

    float alpha = compute_effective_opacity(noise1, noise2, caust1, caust2);
    COLOR = (noise1 + noise2) * alpha + bg;
}
