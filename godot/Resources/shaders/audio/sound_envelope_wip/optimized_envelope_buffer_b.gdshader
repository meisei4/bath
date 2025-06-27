shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/shaders/audio/sound_envelope_wip/utils.gdshaderinc"

uniform vec2      iResolution;
uniform int       iFrame;
uniform sampler2D iChannel0;
uniform sampler2D iChannel1 : hint_screen_texture;

#define LINE_RENDER_WIDTH 0.90
#define LINE_RENDER_MARGIN                                                                                             \
    (LINE_RENDER_WIDTH * 2.0) // amount of thickness (in fragment size/single pixel) that surrounds both sides of the
                              // wave signal lines
#define MAX_DISTANCE 1e6 // some stupid number to just initialize the min distance to closest wave signal logic

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define BUFFER_A_1D_ROW 0.0

#define FRAMES_PER_PROPAGATION                                                                                         \
    4.0 // TODO: THIS IS NOT AT ALL COOL OR WHAT YOU WANT, you have to tune it to work with the injection interval very
        // sensitiviely

float get_envelope_from_buffer(int history_row, int segment_index) {
    float texture_u = (float(segment_index) + 0.5) / float(DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS);
    return texture(iChannel0, vec2(texture_u, BUFFER_A_1D_ROW)).r;
}

vec2 project_centered_envelope(vec3 envelope_coordinate) {
    vec2 raw_projected_coordinate = project_envelope(envelope_coordinate);
    vec2 grid_center              = compute_envelope_grid_center();
    vec2 screen_center            = iResolution.xy * 0.5;
    return raw_projected_coordinate + (screen_center - grid_center);
}

vec4 propagate_envelope_injection_snapshot_isometrically(vec2 uv, vec2 uv_shift) {
    vec4 envelope_fragment = texture(iChannel1, uv + uv_shift);
    return envelope_fragment;
}

void fragment() {
    vec2 uv       = FRAGCOORD.xy / iResolution.xy;
    vec2 uv_shift = compute_isometric_uv_shift(iResolution.xy);

    float should_propagate = step(0.5, 1.0 - step(0.001, mod(float(iFrame), float(FRAMES_PER_PROPAGATION))));

    vec4  propagated_fragment    = texture(iChannel1, uv + uv_shift * should_propagate);
    float current_amplitude      = propagated_fragment.r;
    float current_snapshot_index = propagated_fragment.a + should_propagate;

    if (current_snapshot_index >= float(NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS)) {
        current_amplitude      = 0.0;
        current_snapshot_index = 0.0;
    }
    float new_intensity = 0.0;
    if (should_propagate > 0.5) {
        float closest_distance = MAX_DISTANCE;
        for (int segment_index = 0; segment_index < DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS - 1;
            segment_index++) {
            float envelope_left  = get_envelope_from_buffer(0, segment_index);
            float envelope_right = get_envelope_from_buffer(0, segment_index + 1);

            vec3 envelope_coordinate_left  = vec3(float(segment_index), envelope_left, 0.);
            vec3 envelope_coordinate_right = vec3(float(segment_index + 1), envelope_right, 0.);

            vec2 screen_position_left  = project_centered_envelope(envelope_coordinate_left);
            vec2 screen_position_right = project_centered_envelope(envelope_coordinate_right);

            float distance_to_wave_line = distance_to_line(FRAGCOORD.xy, screen_position_left, screen_position_right);
            closest_distance            = min(closest_distance, distance_to_wave_line);
        }
        new_intensity = 1.0 - smoothstep(LINE_RENDER_WIDTH, LINE_RENDER_MARGIN, closest_distance);
    }

    if (new_intensity > 0.005) {
        current_amplitude      = new_intensity;
        current_snapshot_index = 0.0;
    }
    COLOR = vec4(current_amplitude, current_amplitude, current_amplitude, current_snapshot_index);
}
