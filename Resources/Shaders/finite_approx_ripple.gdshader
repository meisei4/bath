//finite_approx_ripple
shader_type canvas_item;

#include "res://Resources/Shaders/constants.gdshaderinc"
uniform float iTime;
uniform vec2 iResolution;
uniform vec3 iMouse;
uniform sampler2D iChannel0 : hint_screen_texture;
//IDEAL ADJUSTABLE PARAMETERS:
//EFFECTIVE DOMAIN: [0.5, 20.0] UNITS: multiplier against base 1.0 size (percentage of screen resolution)
#define RIPPLE_SCALE 0.75 // to scale the size of the ripple
//EFFECTIVE DOMAIN: [0.1, 1.0] UNITS: multiplier against base 1x speed
#define SPEED_FACTOR 0.5 // to slow down ripple
//EFFECTIVE DOMAIN: [-0.34, 1.0] TRY -0.34 to break things,
#define PROPAGATION_INTENSITY 1.0 //to speed up the ripple
//EFFECTIVE DOMAIN: [0.025, 0.5], UNITS: percentage of canvas resolution
#define IMPULSE_WAVE_WIDTH 0.025 //to adjust the wave front width

//BASE PARAMETERS:
#define BASE_SAMPLE_STEP 0.005
#define BASE_IMPULSE_STRENGTH -0.15 // Base impulse strength (height units per frame)
#define BASE_PROPAGATION 1.0 // EFFECTIVE_PROPAGATION to vary between 1.0 and 1.15,
// TODO: tie this with the advent
#define BASE_DAMPENING 0.99 // EFFECTIVE_DAMPENING to vary between 95% and 15% of 95%

//EFFECTIVE PARAMETERS DERVIED FROM IDEAL PARAMETERS AND BASE PARAMETERS:
#define EFFECTIVE_SAMPLE_STEP (BASE_SAMPLE_STEP * SPEED_FACTOR)
#define EFFECTIVE_RIPPLE_SCALE (RIPPLE_SCALE / sqrt(SPEED_FACTOR))
#define IMPULSE_INNER_RADIUS  (0.025 * EFFECTIVE_RIPPLE_SCALE)
#define IMPULSE_OUTER_RADIUS (IMPULSE_INNER_RADIUS + IMPULSE_WAVE_WIDTH * EFFECTIVE_RIPPLE_SCALE)
#define EFFECTIVE_PROPAGATION (BASE_PROPAGATION + 0.15 * PROPAGATION_INTENSITY)  // Ranges from 1.0 to 1.15.
#define EFFECTIVE_DAMPENING (BASE_DAMPENING - 0.15 * PROPAGATION_INTENSITY) // Ranges from 95% down to 15% of 95%

float sample_height1(sampler2D tex, vec2 uv) {
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0)
        return 0.0;
    return textureLod(tex, uv, 0.0).r;
}

float sample_height(sampler2D tex, vec2 uv) {
    return texture(tex, clamp(uv, vec2(0.0), vec2(1.0))).r;
}

void fragment() {
    vec2 uv = FRAGCOORD.xy / iResolution;

    float adjusted_sample_step = EFFECTIVE_SAMPLE_STEP * (iResolution.x / iResolution.y);
    vec2 neighbor_offset_x = vec2(EFFECTIVE_SAMPLE_STEP, 0.0);
    vec2 neighbor_offset_y = vec2(0.0, adjusted_sample_step);
    // Retrieve previous frame height data:
    //   prev_height: height from 1 frame ago (red channel)
    //   prev_prev_height: height from 2 frames ago (green channel)
    vec2 prev_heights = texture(iChannel0, uv).rg;
    float height_left   = sample_height(iChannel0, UV - neighbor_offset_x);
    float height_right  = sample_height(iChannel0, UV + neighbor_offset_x);
    float height_bottom = sample_height(iChannel0, UV - neighbor_offset_y);
    float height_top    = sample_height(iChannel0, UV + neighbor_offset_y);

    vec2 mouse_position = iMouse.xy / iResolution.xy;
    //vec2 prev_mouse_position = texture(iChannel0, uv).ba;
    float mouse_impulse = 0.0;

    if (iMouse.z > 0.0) {
        //BASIC
        float uv_distance_from_mouse = length(mouse_position - UV);
        mouse_impulse = BASE_IMPULSE_STRENGTH * smoothstep(IMPULSE_OUTER_RADIUS, IMPULSE_INNER_RADIUS, uv_distance_from_mouse);
    }
    float avg_neighbor_height = (height_left + height_right + height_top + height_bottom) / 4.0;
    float new_height = prev_heights.r + EFFECTIVE_PROPAGATION * (avg_neighbor_height - prev_heights.g);
    new_height *= EFFECTIVE_DAMPENING;
    new_height += mouse_impulse;
    COLOR = vec4(new_height, prev_heights.r, mouse_position.x, mouse_position.y);
}
