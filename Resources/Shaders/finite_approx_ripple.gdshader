shader_type canvas_item;
render_mode blend_disabled; // TODO you have to do this if you want to send feedback vals in the alpha channel idiot

#include "res://Resources/Shaders/constants.gdshaderinc"

uniform vec3 iMouse;
uniform vec2 iResolution;
uniform sampler2D iChannel0: hint_screen_texture;

//IDEAL ADJUSTABLE PARAMETERS:
//EFFECTIVE DOMAIN: [0.5, 20.0] UNITS: multiplier against base 1.0 size (percentage of screen resolution)
#define RIPPLE_SCALE 0.25 // to scale the size of the ripple
//EFFECTIVE DOMAIN: [0.1, 1.0] UNITS: multiplier against base 1x speed
#define SPEED_FACTOR 1.0 // to slow down ripple
//EFFECTIVE DOMAIN: [-0.34, 1.0] TRY -0.34 to break things,
#define PROPAGATION_INTENSITY -0.1 //to speed up the ripple
//EFFECTIVE DOMAIN: [0.025, 0.5], UNITS: percentage of canvas resolution
#define IMPULSE_WAVE_WIDTH 0.025 //to adjust the wave front width

//BASE PARAMETERS:
#define BASE_SAMPLE_STEP 0.005
#define BASE_IMPULSE_STRENGTH -0.015 // Base impulse strength (height units per frame)
#define BASE_PROPAGATION 1.0 // EFFECTIVE_PROPAGATION to vary between 1.0 and 1.15,
// TODO: tie this with the advent
#define BASE_DAMPENING 0.80 // EFFECTIVE_DAMPENING to vary between 95% and 15% of 95%

//EFFECTIVE PARAMETERS DERVIED FROM IDEAL PARAMETERS AND BASE PARAMETERS:
#define EFFECTIVE_SAMPLE_STEP (BASE_SAMPLE_STEP * SPEED_FACTOR)
#define EFFECTIVE_RIPPLE_SCALE (RIPPLE_SCALE / sqrt(SPEED_FACTOR))
#define IMPULSE_INNER_RADIUS  (0.025 * EFFECTIVE_RIPPLE_SCALE)
#define IMPULSE_OUTER_RADIUS (IMPULSE_INNER_RADIUS + IMPULSE_WAVE_WIDTH * EFFECTIVE_RIPPLE_SCALE)
#define EFFECTIVE_PROPAGATION (BASE_PROPAGATION + 0.15 * PROPAGATION_INTENSITY)  // Ranges from 1.0 to 1.15.
#define EFFECTIVE_DAMPENING (BASE_DAMPENING - 0.15 * PROPAGATION_INTENSITY) // Ranges from 95% down to 15% of 95%

float sample_height(sampler2D tex, vec2 uv) {
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0)
        return 0.0;
    return texture(tex, uv).r;
}

float compute_wavefront(vec2 uv, vec2 mouse_position, vec2 prev_mouse_position) {
    float uv_distance_from_mouse = length(mouse_position - uv);
    //float audio_factor = get_audio_scale_factor();
    float audio_effective_inner_radius = IMPULSE_INNER_RADIUS;// * audio_factor;
    float audio_effective_outer_radius = IMPULSE_OUTER_RADIUS;// * audio_factor;
    float radial_impulse = BASE_IMPULSE_STRENGTH * smoothstep(audio_effective_outer_radius, audio_effective_inner_radius, uv_distance_from_mouse);// * audio_factor;
    vec2 mouse_velocity = mouse_position - prev_mouse_position;
    float movement = length(mouse_velocity);
    float directional_factor = 1.0;

    if (movement > 0.001) {
        vec2 mouse_direction = normalize(mouse_velocity);
        vec2 to_fragment = normalize(uv - mouse_position);
        directional_factor = step(0.0, dot(mouse_direction, to_fragment));
    }

    return radial_impulse * directional_factor;
}

void fragment(){
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    //vec2 uv = frag_coord / iResolution.xy;
    // Adjust the y sample offset to compensate for non-square resolutions.
    // Without this, a fixed UV step would correspond to different pixel distances in x and y,
    // resulting in anisotropic (e.g. elliptical) ripples.
    float adjusted_sample_step = EFFECTIVE_SAMPLE_STEP * (iResolution.x / iResolution.y);
    vec2 neighbor_offset_x = vec2(EFFECTIVE_SAMPLE_STEP, 0.0);
    vec2 neighbor_offset_y = vec2(0.0, adjusted_sample_step);
    // Retrieve previous frame height data:
    //   prev_height: height from 1 frame ago (red channel)
    //   prev_prev_height: height from 2 frames ago (green channel)
    vec2 prev_heights = texture(iChannel0, uv).rg;
    float height_left   = sample_height(iChannel0, uv - neighbor_offset_x);
    float height_right  = sample_height(iChannel0, uv + neighbor_offset_x);
    float height_bottom = sample_height(iChannel0, uv - neighbor_offset_y);
    float height_top    = sample_height(iChannel0, uv + neighbor_offset_y);

    vec2 mouse_position = iMouse.xy / iResolution.xy;
    vec2 prev_mouse_position = texture(iChannel0, uv).ba;
    float mouse_impulse = 0.0;

    if (iMouse.z > 0.0) {
        //float uv_distance_from_mouse = length(mouse_position - uv);
        //mouse_impulse = BASE_IMPULSE_STRENGTH * smoothstep(IMPULSE_OUTER_RADIUS, IMPULSE_INNER_RADIUS, uv_distance_from_mouse);

        //WAVEFRONT ONLY
        mouse_impulse = compute_wavefront(uv, mouse_position, prev_mouse_position);
    }
    float avg_neighbor_height = (height_left + height_right + height_top + height_bottom) / 4.0;
    float new_height = prev_heights.r + EFFECTIVE_PROPAGATION * (avg_neighbor_height - prev_heights.g);
    new_height *= EFFECTIVE_DAMPENING;
    new_height += mouse_impulse;

    vec2 gradient = vec2(height_right - height_left, height_top - height_bottom);
    vec2 mouse_velocity = mouse_position - prev_mouse_position;
    float advection = 0.0;
    if (length(mouse_velocity) > 0.0001) {
        vec2 mouse_direction = normalize(mouse_velocity);
        advection = dot(mouse_direction, gradient);
    }
    //TODO: MAKE THIS SCALE WITH THE DAMPENING!!!
    new_height += -0.075 * advection;
    COLOR = vec4(new_height, prev_heights.r, mouse_position.x, mouse_position.y);
    //frag_color = vec4(new_height, prev_heights.r, mouse_position.x, mouse_position.y);
}
