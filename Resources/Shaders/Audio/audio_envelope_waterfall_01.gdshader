shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/Audio/audio_plotting.gdshaderinc"

uniform vec2 iResolution;
uniform sampler2D iChannel0; //samples the output of audio_feedback_envelope

#define LINE_RENDER_MARGIN 0.23        // amount of thickness (in fragment size/single pixel) that surrounds both sides of the wave signal lines
#define MAX_DISTANCE 1e6               // some stupid number to just initialize the min distance to closest wave signal logic

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)

const float line_render_width = 0.75;

float get_envelope_from_buffer(int history_row, int bin_index) {
    float texture_v = 1.0 - (float(history_row) + 0.5) / float(NUMBER_OF_HISTORY_ROWS);
    float texture_u = (float(bin_index) + 0.5) / float(NUMBER_OF_BINS);
    return texture(iChannel0, vec2(texture_u, texture_v)).r;
}

// Offsets a projected envelope coordinate so the full grid is centered on screen.
vec2 project_centered_envelope(vec3 envelope_coordinate) {
    vec2 raw_projected_coordinate = project_envelope(envelope_coordinate);
    vec2 grid_center = compute_envelope_grid_center();
    vec2 screen_center = iResolution.xy * 0.5;
    return raw_projected_coordinate + (screen_center - grid_center);
}

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    float closest_distance = MAX_DISTANCE;

    // Loop over each history row (each envelope capture).
    for (int history_row = 0; history_row < NUMBER_OF_HISTORY_ROWS; history_row++) {
        // Loop over each pair of adjacent bins.
        for (int bin_index = 0; bin_index < NUMBER_OF_BINS - 1; bin_index++) {
            float envelope_left = get_envelope_from_buffer(history_row, bin_index);
            float envelope_right = get_envelope_from_buffer(history_row, bin_index + 1);

            vec3 envelope_coordinate_left = vec3(float(bin_index), envelope_left, float(history_row));
            vec3 envelope_coordinate_right = vec3(float(bin_index + 1), envelope_right, float(history_row));

            vec2 screen_position_left = project_centered_envelope(envelope_coordinate_left);
            vec2 screen_position_right = project_centered_envelope(envelope_coordinate_right);

            float distance_to_wave_line = distance_to_line(FRAGCOORD.xy, screen_position_left, screen_position_right);
            //float distance_to_wave_line = distance_to_line(frag_coord.xy, screen_position_left, screen_position_right);

            closest_distance = min(closest_distance, distance_to_wave_line);
        }
    }

    float intensity = 1.0 - smoothstep(line_render_width, line_render_width * 2.0, closest_distance);
    //frag_color = vec4(vec3(intensity), 1.0);
    COLOR = vec4(vec3(intensity), 1.0);
}
