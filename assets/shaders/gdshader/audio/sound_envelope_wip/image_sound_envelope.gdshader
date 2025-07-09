shader_type canvas_item;
render_mode blend_disabled;

#include "res://assets/shaders/gdshader/audio/sound_envelope_wip/utils.gdshaderinc"

uniform vec2      iResolution;
uniform sampler2D iChannel0;

#define LINE_RENDER_WIDTH 0.25
#define LINE_RENDER_MARGIN                                                                                             \
    (LINE_RENDER_WIDTH * 2.0) // amount of thickness (in fragment size/single pixel) that surrounds both sides of the
                              // wave signal lines
#define MAX_DISTANCE 1e6 // some stupid number to just initialize the min distance to closest wave signal logic

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)

float get_envelope_from_buffer(int history_row, int segment_index) {
    float texture_v = 1.0 - (float(history_row) + 0.5) / float(NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS);
    float texture_u = (float(segment_index) + 0.5) / float(DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS);
    return texture(iChannel0, vec2(texture_u, texture_v)).r;
}

vec2 project_centered_envelope(vec3 envelope_coordinate) {
    vec2 raw_projected_coordinate = project_envelope(envelope_coordinate);
    vec2 grid_center              = compute_envelope_grid_center();
    vec2 screen_center            = iResolution.xy * 0.5;
    return raw_projected_coordinate + (screen_center - grid_center);
}

void fragment() {
    float closest_distance = MAX_DISTANCE;
    for (int history_row = 0; history_row < NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS; history_row++) {
        for (int segment_index = 0; segment_index < DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS - 1;
            segment_index++) {
            float envelope_left  = get_envelope_from_buffer(history_row, segment_index);
            float envelope_right = get_envelope_from_buffer(history_row, segment_index + 1);

            vec3 envelope_coordinate_left  = vec3(float(segment_index), envelope_left, float(history_row));
            vec3 envelope_coordinate_right = vec3(float(segment_index + 1), envelope_right, float(history_row));

            vec2 screen_position_left  = project_centered_envelope(envelope_coordinate_left);
            vec2 screen_position_right = project_centered_envelope(envelope_coordinate_right);

            float distance_to_wave_line = distance_to_line(FRAGCOORD.xy, screen_position_left, screen_position_right);
            closest_distance            = min(closest_distance, distance_to_wave_line);
        }
    }

    float intensity = 1.0 - smoothstep(LINE_RENDER_WIDTH, LINE_RENDER_MARGIN, closest_distance);
    COLOR           = vec4(vec3(intensity), 1.0);
}
