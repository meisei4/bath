shader_type canvas_item;

#include "res://assets/shaders/gdshader/ice_sheets/projections.gdshaderinc"
#include "res://assets/shaders/gdshader/water/constants.gdshaderinc"
uniform vec2 iResolution;

uniform sampler2D iChannel0 : repeat_enable;
uniform sampler2D iChannel1 : repeat_enable;
uniform sampler2D iChannel2 : repeat_enable;
uniform sampler2D iChannel3;

// #define PIXELATE_UV
#define SHOW_NOISE_DISP_MAP_1
#define SHOW_NOISE_DISP_MAP_2
#define SHOW_CAUSTICS_DISP_MAP_1
#define SHOW_CAUSTICS_DISP_MAP_2
#define SHOW_BACKGROUND
#define RIPPLE_EFFECT

vec2 pixelate_uv(vec2 uv) {
    return floor(uv * vec2(VIRTUAL_DS_RES_X, VIRTUAL_DS_RES_Y)) / vec2(VIRTUAL_DS_RES_X, VIRTUAL_DS_RES_Y);
}

vec2 scroll_displacement_map(vec2 uv, vec2 velocity, vec2 positional_offset) {
    return uv + TIME * velocity + positional_offset;
}

vec4 sample_disp_map(sampler2D tex, vec2 uv, vec2 velocity, vec2 positional_offset, float intensity_factor) {
    vec2  offset_uv    = scroll_displacement_map(uv, velocity, positional_offset);
    float noise_value  = texture(tex, offset_uv).r;
    float scaled_noise = noise_value * intensity_factor;
    return vec4(scaled_noise, scaled_noise, scaled_noise, 1.0);
}

vec4 sample_background_with_disp_map(sampler2D tex, vec2 uv, vec4 disp_map, float warp_factor) {
    vec2 bg_uv = uv + (disp_map.r * warp_factor);
    return texture(tex, bg_uv);
}

vec2 compute_ripple_offset(vec2 uv) {
    float height_center  = texture(iChannel3, uv).r;
    float height_left    = texture(iChannel3, uv - vec2(NORMAL_SAMPLE_OFFSET, 0.0)).r;
    float height_right   = texture(iChannel3, uv + vec2(NORMAL_SAMPLE_OFFSET, 0.0)).r;
    float height_up      = texture(iChannel3, uv + vec2(0.0, NORMAL_SAMPLE_OFFSET)).r;
    float height_down    = texture(iChannel3, uv - vec2(0.0, NORMAL_SAMPLE_OFFSET)).r;
    float d_x            = height_right - height_left;
    float d_y            = height_up - height_down;
    vec3  raw_normal     = vec3(d_x, d_y, NORMAL_Z_SCALE * NORMAL_SAMPLE_OFFSET);
    vec3  surface_normal = normalize(raw_normal);
    vec3  refracted      = refract(INCIDENT_DIRECTION, surface_normal, REFRACTION_INDEX_RATIO);
    return refracted.xy;
}

float compute_effective_opacity(
    vec4 noise_disp_map_1, vec4 noise_disp_map_2, vec4 caustics_disp_map_1, vec4 caustics_disp_map_2) {
    float noise_disp_maps_grayscale_intensity_sum = noise_disp_map_1.r + noise_disp_map_2.r;
    float all_disp_maps_grayscale_intensity_sum
        = noise_disp_map_1.r + noise_disp_map_2.r + caustics_disp_map_1.r + caustics_disp_map_2.r;

    float alpha = BLURRY_ALPHA;

    if (noise_disp_maps_grayscale_intensity_sum > NOISE_DISP_INDUCED_INTENSITY_THRESHOLD) {
        alpha = NORMAL_ALPHA;
    }
    if (all_disp_maps_grayscale_intensity_sum > ALL_DISP_MAP_INDUCED_INTENSITY_THRESHOLD) {
        alpha = FULL_ALPHA;
    }
    return alpha;
}

#define PARALLAX_ZOOM 16.0
void fragment() {
    vec2 uv                  = vec2(FRAGCOORD.x, iResolution.y - FRAGCOORD.y) / iResolution.xy;
    vec4 noise_disp_map_1    = vec4(0.0);
    vec4 noise_disp_map_2    = vec4(0.0);
    vec4 caustics_disp_map_1 = vec4(0.0);
    vec4 caustics_disp_map_2 = vec4(0.0);
    vec4 background          = vec4(0.0);

#ifdef PIXELATE_UV
    uv = pixelate_uv(uv);
#endif

#ifdef RIPPLE_EFFECT
    uv += compute_ripple_offset(uv);
#endif

#ifdef SHOW_NOISE_DISP_MAP_1
    noise_disp_map_1 = sample_disp_map(iChannel0, uv, NOISE_DISP_MAP_1_SCROLL_VELOCITY, NOISE_DISP_MAP_1_INITIAL_OFFSET,
        NOISE_DISP_MAP_DIMMING_FACTOR);
#endif

#ifdef SHOW_NOISE_DISP_MAP_2
    noise_disp_map_2 = sample_disp_map(iChannel0, uv, NOISE_DISP_MAP_2_SCROLL_VELOCITY, NOISE_DISP_MAP_2_INITIAL_OFFSET,
        NOISE_DISP_MAP_DIMMING_FACTOR);
#endif

#ifdef SHOW_CAUSTICS_DISP_MAP_1
    caustics_disp_map_1 = sample_disp_map(
        iChannel2, uv, CAUSTICS_DISP_MAP_1_SCROLL_VELOCITY, ZERO_POSITIONAL_OFFSET, CAUSTICS_DISP_MAP_DIMMING_FACTOR);
#endif

#ifdef SHOW_CAUSTICS_DISP_MAP_2
    caustics_disp_map_2 = sample_disp_map(
        iChannel2, uv, CAUSTICS_DISP_MAP_2_SCROLL_VELOCITY, ZERO_POSITIONAL_OFFSET, CAUSTICS_DISP_MAP_DIMMING_FACTOR);
#endif

#ifdef SHOW_BACKGROUND
    background = sample_background_with_disp_map(iChannel1, uv, noise_disp_map_1, BACKGROUND_DISP_WARP_FACTOR);
#endif

    float alpha
        = compute_effective_opacity(noise_disp_map_1, noise_disp_map_2, caustics_disp_map_1, caustics_disp_map_2);

    COLOR = (noise_disp_map_1 + noise_disp_map_2) * alpha + background;
}
