shader_type canvas_item;

#include "res://Resources/Shaders/constants.gdshaderinc"

uniform float iTime;
uniform vec2 iResolution;
uniform vec3 iMouse;
uniform sampler2D iChannel0; // Previous simulation state (feedback texture)
uniform sampler2D iChannel1; // Background image texture

// --- Simulation Parameters ---
#define RIPPLE_SCALE 0.5
#define SPEED_FACTOR 1.0
#define PROPAGATION_INTENSITY 0.0
#define IMPULSE_WAVE_WIDTH 0.025

#define BASE_SAMPLE_STEP 0.005
#define BASE_IMPULSE_STRENGTH -0.015
#define BASE_PROPAGATION 1.0
#define BASE_DAMPENING 0.95

#define EFFECTIVE_SAMPLE_STEP (BASE_SAMPLE_STEP * SPEED_FACTOR)
#define EFFECTIVE_RIPPLE_SCALE (RIPPLE_SCALE / sqrt(SPEED_FACTOR))
#define IMPULSE_INNER_RADIUS  (0.025 * EFFECTIVE_RIPPLE_SCALE)
#define IMPULSE_OUTER_RADIUS (IMPULSE_INNER_RADIUS + IMPULSE_WAVE_WIDTH * EFFECTIVE_RIPPLE_SCALE)
#define EFFECTIVE_PROPAGATION (BASE_PROPAGATION + 0.15 * PROPAGATION_INTENSITY)
#define EFFECTIVE_DAMPENING (BASE_DAMPENING - 0.15 * PROPAGATION_INTENSITY)

float sample_height(sampler2D tex, vec2 uv) {
    // Handle boundaries as needed:
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0)
        return 0.0;
    return texture(tex, uv).r;
}

void fragment() {
    // --- SIMULATION UPDATE ---
    // Sample neighboring heights from previous state:
    vec2 neighbor_offset_x = vec2(EFFECTIVE_SAMPLE_STEP, 0.0);
    vec2 neighbor_offset_y = vec2(0.0, EFFECTIVE_SAMPLE_STEP);
    float prev_height = texture(iChannel0, UV).a; // only stored in a now
    float height_left   = sample_height(iChannel0, UV - neighbor_offset_x);
    float height_right  = sample_height(iChannel0, UV + neighbor_offset_x);
    float height_bottom = sample_height(iChannel0, UV - neighbor_offset_y);
    float height_top    = sample_height(iChannel0, UV + neighbor_offset_y);
    
    float mouse_impulse = 0.0;
    if (iMouse.z > 0.0) {
        // Compute the wavefront based on mouse movement:
        float uv_distance_from_mouse = length(iMouse.xy - UV);
        mouse_impulse = BASE_IMPULSE_STRENGTH * smoothstep(IMPULSE_OUTER_RADIUS, IMPULSE_INNER_RADIUS, uv_distance_from_mouse);
    }
    
    float avg_neighbor_height = (height_left + height_right + height_top + height_bottom) / 4.0;
    float new_height = prev_height + EFFECTIVE_PROPAGATION * (avg_neighbor_height - prev_height);
    new_height *= EFFECTIVE_DAMPENING;
    new_height += mouse_impulse;
    
    vec3 normal = normalize(vec3(
        sample_height(iChannel0, UV + vec2(NORMAL_SAMPLE_OFFSET, 0.0)) - sample_height(iChannel0, UV - vec2(NORMAL_SAMPLE_OFFSET, 0.0)),
        sample_height(iChannel0, UV + vec2(0.0, NORMAL_SAMPLE_OFFSET)) - sample_height(iChannel0, UV - vec2(0.0, NORMAL_SAMPLE_OFFSET)),
        2.0 * NORMAL_SAMPLE_OFFSET
    ));
    
    vec2 refracted_offset = refract(INCIDENT_DIRECTION, normal, REFRACTION_INDEX_RATIO).xy;
    vec2 bg_uv = UV + refracted_offset * 0.05;
    vec4 background = texture(iChannel1, bg_uv);
    COLOR = vec4(background.rgb, new_height); // pass the current height in the alpha channel
}
