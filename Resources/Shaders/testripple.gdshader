shader_type canvas_item;

#include "res://Resources/Shaders/constants.gdshaderinc"

uniform float iTime;
uniform vec2 iResolution;
uniform vec3 iMouse;

uniform sampler2D iChannel0; // current heightmap (updated by GDScript)
uniform sampler2D iChannel2; // background image (rocks.jpg)
uniform sampler2D iChannel3; // noise texture
uniform sampler2D iChannel4; // pebbles texture

#define SHOW_BACKGROUND
#define RIPPLE_EFFECT
//#define SHOW_NOISE_DISP_MAP_1
//#define SHOW_NOISE_DISP_MAP_2
//#define SHOW_CAUSTICS_DISP_MAP_1
//#define SHOW_CAUSTICS_DISP_MAP_2

#define RIPPLE_SCALE 0.25
#define SPEED_FACTOR 1.0
#define PROPAGATION_INTENSITY 1.0
#define IMPULSE_WAVE_WIDTH 0.025

#define BASE_SAMPLE_STEP 0.005
#define BASE_IMPULSE_STRENGTH -0.15

#define EFFECTIVE_SAMPLE_STEP (BASE_SAMPLE_STEP * SPEED_FACTOR)
#define EFFECTIVE_RIPPLE_SCALE (RIPPLE_SCALE / sqrt(SPEED_FACTOR))
#define IMPULSE_INNER_RADIUS  (0.025 * EFFECTIVE_RIPPLE_SCALE)
#define IMPULSE_OUTER_RADIUS (IMPULSE_INNER_RADIUS + IMPULSE_WAVE_WIDTH * EFFECTIVE_RIPPLE_SCALE)

float sample_height(sampler2D tex, vec2 uv) {
    return texture(tex, clamp(uv, vec2(0.0), vec2(1.0))).r;
}

vec2 compute_ripple_offset(vec2 uv) {
    float step = NORMAL_SAMPLE_OFFSET;
    float h_c = sample_height(iChannel0, uv);
    float h_l = sample_height(iChannel0, uv - vec2(step, 0.0));
    float h_r = sample_height(iChannel0, uv + vec2(step, 0.0));
    float h_u = sample_height(iChannel0, uv + vec2(0.0, step));
    float h_d = sample_height(iChannel0, uv - vec2(0.0, step));

    float dx = h_r - h_l;
    float dy = h_u - h_d;

    vec3 raw_normal = vec3(dx, dy, NORMAL_Z_SCALE * step);
    vec3 surface_normal = normalize(raw_normal);
    vec3 refracted = refract(INCIDENT_DIRECTION, surface_normal, REFRACTION_INDEX_RATIO);
    return refracted.xy;
}

vec2 scroll_displacement_map(vec2 uv, vec2 velocity, vec2 offset) {
    return uv + iTime * velocity + offset;
}

vec4 sample_disp_map(
    sampler2D tex,
    vec2 uv,
    vec2 velocity,
    vec2 offset,
    float factor
) {
    vec2 offset_uv = scroll_displacement_map(uv, velocity, offset);
    float v = texture(tex, offset_uv).r;
    float scaled = v * factor;
    return vec4(scaled, scaled, scaled, 1.0);
}

vec4 sample_background_with_disp_map(
    sampler2D tex,
    vec2 uv,
    vec4 disp_map,
    float warp
) {
    vec2 displaced = uv + (disp_map.r * warp);
    return texture(tex, displaced);
}

float compute_effective_opacity(
    vec4 n1, vec4 n2,
    vec4 c1, vec4 c2
) {
    float noise_sum = n1.r + n2.r;
    float all_sum = noise_sum + c1.r + c2.r;

    float alpha = BLURRY_ALPHA;

    if (noise_sum > NOISE_DISP_INDUCED_INTENSITY_THRESHOLD) {
        alpha = NORMAL_ALPHA;
    }
    if (all_sum > ALL_DISP_MAP_INDUCED_INTENSITY_THRESHOLD) {
        alpha = FULL_ALPHA;
    }

    return alpha;
}

void fragment() {
    vec2 uv = FRAGCOORD.xy / iResolution;

    #ifdef RIPPLE_EFFECT
        uv += compute_ripple_offset(uv);
    #endif

    vec4 noise_disp_map_1 = vec4(0.0);
    vec4 noise_disp_map_2 = vec4(0.0);
    vec4 caustics_disp_map_1 = vec4(0.0);
    vec4 caustics_disp_map_2 = vec4(0.0);
    vec4 background = vec4(0.0);

    #ifdef SHOW_NOISE_DISP_MAP_1
        noise_disp_map_1 = sample_disp_map(
            iChannel3, uv,
            NOISE_DISP_MAP_1_SCROLL_VELOCITY,
            NOISE_DISP_MAP_1_INITIAL_OFFSET,
            NOISE_DISP_MAP_DIMMING_FACTOR
        );
    #endif

    #ifdef SHOW_NOISE_DISP_MAP_2
        noise_disp_map_2 = sample_disp_map(
            iChannel3, uv,
            NOISE_DISP_MAP_2_SCROLL_VELOCITY,
            NOISE_DISP_MAP_2_INITIAL_OFFSET,
            NOISE_DISP_MAP_DIMMING_FACTOR
        );
    #endif

    #ifdef SHOW_CAUSTICS_DISP_MAP_1
        caustics_disp_map_1 = sample_disp_map(
            iChannel4, uv,
            CAUSTICS_DISP_MAP_1_SCROLL_VELOCITY,
            ZERO_POSITIONAL_OFFSET,
            CAUSTICS_DISP_MAP_DIMMING_FACTOR
        );
    #endif

    #ifdef SHOW_CAUSTICS_DISP_MAP_2
        caustics_disp_map_2 = sample_disp_map(
            iChannel4, uv,
            CAUSTICS_DISP_MAP_2_SCROLL_VELOCITY,
            ZERO_POSITIONAL_OFFSET,
            CAUSTICS_DISP_MAP_DIMMING_FACTOR
        );
    #endif

    #ifdef SHOW_BACKGROUND
        background = sample_background_with_disp_map(
            iChannel2,
            uv,
            noise_disp_map_1,
            BACKGROUND_DISP_WARP_FACTOR
        );
    #endif

    float alpha = compute_effective_opacity(
        noise_disp_map_1,
        noise_disp_map_2,
        caustics_disp_map_1,
        caustics_disp_map_2
    );

    COLOR = (noise_disp_map_1 + noise_disp_map_2 + caustics_disp_map_1 + caustics_disp_map_2) * alpha + background;
}
