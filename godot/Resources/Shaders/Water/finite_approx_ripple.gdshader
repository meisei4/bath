shader_type canvas_item;
render_mode blend_disabled; // TODO you have to do this if you want to send feedback vals in the alpha channel idiot

#include "res://Resources/Shaders/Water/constants.gdshaderinc"

uniform vec2 iResolution;
uniform sampler2D iChannel0: hint_screen_texture;

#define MAX_ICEBERGS 24u
//TODO: !!!!If you expect more tiles than you can comfortably pass as a uniform array, you would switch to passing them via a small texture instead.
uniform vec2 iceberg_positions[MAX_ICEBERGS];
uniform vec2 iceberg_velocities[MAX_ICEBERGS];
uniform vec2 iceberg_tile_positions[1024];  //TODO: 1024 this should be just how many INTACT glacier tiles there are at the start of the sim
uniform int cluster_offsets[128];  // pairs: [start_index, end_index, ...]
uniform float tile_size;

//IDEAL ADJUSTABLE PARAMETERS:
//EFFECTIVE DOMAIN: [0.5, 20.0] UNITS: multiplier against base 1.0 size (percentage of screen resolution)
#define RIPPLE_SCALE 0.1 // to scale the size of the ripple
//EFFECTIVE DOMAIN: [0.1, 1.0] UNITS: multiplier against base 1x speed
#define SPEED_FACTOR 0.5 // to slow down ripple
//EFFECTIVE DOMAIN: [-0.34, 1.0] TRY -0.34 to break things,
#define PROPAGATION_INTENSITY 0.0 //to speed up the ripple
//EFFECTIVE DOMAIN: [0.025, 0.5], UNITS: percentage of canvas resolution
#define IMPULSE_WAVE_WIDTH 0.025 //to adjust the wave front width


//BASE PARAMETERS:
#define BASE_SAMPLE_STEP 0.005
#define BASE_IMPULSE_STRENGTH -0.015 // Base impulse strength (height units per frame)
#define BASE_PROPAGATION 1.0 // EFFECTIVE_PROPAGATION to vary between 1.0 and 1.15,
// TODO: tie this with the advent
#define BASE_DAMPENING 0.6 // EFFECTIVE_DAMPENING to vary between 95% and 15% of 95%

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

float compute_wavefront_advection(vec2 uv, vec2 iceberg_position, vec2 iceberg_wavefront_velocity, vec2 gradient) {
    float uv_distance_from_iceberg_wavefront = length(iceberg_position - uv);
    // TODO: Weight the contribution: closer fragments get more effect?????
    float weight = 0.1 - smoothstep(IMPULSE_OUTER_RADIUS, IMPULSE_INNER_RADIUS, uv_distance_from_iceberg_wavefront);
    float advection = 0.0;
    if (length(iceberg_wavefront_velocity) > 0.0001) {
        vec2 wavefront_velocity_direction = normalize(iceberg_wavefront_velocity);
        advection = dot(wavefront_velocity_direction, gradient);
    }
    return advection * weight;
}

float distance_to_tile_boundary(vec2 uv, vec2 iceberg_tile_center_uv, vec2 half_size_uv) {
    vec2 distance_to_iceberg_tile_boundary = abs(uv - iceberg_tile_center_uv) - half_size_uv;
    return max(distance_to_iceberg_tile_boundary.x, distance_to_iceberg_tile_boundary.y);
}

float compute_silhouette_mask_wavefront(vec2 uv, int cluster_index, vec2 cluster_velocity) {
    int start_idx = cluster_offsets[cluster_index * 2];
    int end_idx   = cluster_offsets[cluster_index * 2 + 1];
    vec2 iceberg_cluster_anchor_screen_coordinates = iceberg_positions[cluster_index];
    float min_distance = 999999.0;
    float final_impulse = 0.0;
    for (int i = start_idx; i < end_idx; i++) {
        vec2 local_offset_in_iceberg_cluster_bounding_box_in_screen_coordinates = iceberg_tile_positions[i];
        vec2 iceberg_tile_center_in_screen_coordinates = iceberg_cluster_anchor_screen_coordinates + local_offset_in_iceberg_cluster_bounding_box_in_screen_coordinates + vec2(tile_size * 0.5, tile_size * 0.5);
        vec2 iceberg_tile_center_in_uv_coordinates = iceberg_tile_center_in_screen_coordinates / iResolution;
        vec2 half_tile_size_uv = vec2(tile_size * 0.5) / iResolution;
        float distance_to_tile_boundary = distance_to_tile_boundary(uv, iceberg_tile_center_in_uv_coordinates, half_tile_size_uv);
        //float audio_factor = get_audio_scale_factor();
        float audio_effective_inner_radius = IMPULSE_INNER_RADIUS;// * audio_factor;
        float audio_effective_outer_radius = IMPULSE_OUTER_RADIUS;// * audio_factor;
        float wave_factor = smoothstep(audio_effective_outer_radius, audio_effective_inner_radius, distance_to_tile_boundary);
        vec2 to_fragment = normalize(uv - iceberg_tile_center_in_uv_coordinates);
        float directional_factor = clamp(dot(normalize(cluster_velocity), to_fragment), 0.0, 1.0);
        final_impulse += BASE_IMPULSE_STRENGTH * wave_factor * directional_factor;
    }
    return final_impulse;
}

void fragment(){
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    float adjusted_sample_step = EFFECTIVE_SAMPLE_STEP * (iResolution.x / iResolution.y);
    vec2 neighbor_offset_x = vec2(EFFECTIVE_SAMPLE_STEP, 0.0);
    vec2 neighbor_offset_y = vec2(0.0, adjusted_sample_step);
    vec2 prev_heights = texture(iChannel0, uv).rg;
    float height_left   = sample_height(iChannel0, uv - neighbor_offset_x);
    float height_right  = sample_height(iChannel0, uv + neighbor_offset_x);
    float height_bottom = sample_height(iChannel0, uv - neighbor_offset_y);
    float height_top    = sample_height(iChannel0, uv + neighbor_offset_y);

    float silhouette_impulse = 0.0;
    for (int i = 0; i < iceberg_positions.length(); i++) {
        silhouette_impulse += compute_silhouette_mask_wavefront(uv, i, iceberg_velocities[i]);
    }
    float avg_neighbor_height = (height_left + height_right + height_top + height_bottom) / 4.0;
    float new_height = prev_heights.r + EFFECTIVE_PROPAGATION * (avg_neighbor_height - prev_heights.g);
    new_height *= EFFECTIVE_DAMPENING;
    new_height += silhouette_impulse;
    vec2 gradient = vec2(height_right - height_left, height_top - height_bottom);
    float total_advection = 0.0;
    for (int i = 0; i < iceberg_positions.length(); i++) {
        vec2 iceberg_position_uv = iceberg_positions[i] / iResolution.xy;
        total_advection += compute_wavefront_advection(uv, iceberg_position_uv, iceberg_velocities[i], gradient);
    }
    new_height += -0.070 * total_advection;
    COLOR = vec4(new_height, prev_heights.r, 0.0, 1.0);
    //frag_color = vec4(new_height, prev_heights.r, 0.0, 1.0);
}
