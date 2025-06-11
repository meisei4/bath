#define DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS 96
#define DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS_F float(DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS)
#define NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS 5
#define NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS_F float(NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS)
#define AMPLITUDE_SCALE 100.0
#define ISOMETRIC_ZOOM 3.0
#define ROW_SPACING 16.0 //THE FUCK THIS CAUSES ARTIFACTS IN THE CLEARING AFTER 5 ROWS IF ITS 12 or something

#define UNUSED_FEEDBACK_CHANNEL 0.0


vec2 project_envelope(vec3 envelope_coordinate) {
    float segment_index   = envelope_coordinate.x;
    float envelope_value  = envelope_coordinate.y;
    float history_row     = envelope_coordinate.z;
    float effective_row   = history_row * ROW_SPACING;
    float projected_x = segment_index - effective_row;
    float projected_y = (segment_index + effective_row) * 0.5 - envelope_value * AMPLITUDE_SCALE;
    return vec2(projected_x, projected_y) * ISOMETRIC_ZOOM;
}

vec2 compute_isometric_uv_shift(vec2 resolution_uniform) {
    vec2 proj0 = project_envelope(vec3(0.0, 0.0, 1.0));
    vec2 proj1 = project_envelope(vec3(0.0, 0.0, 0.0));
    vec2 projected_diff = proj1 - proj0;
    return projected_diff / resolution_uniform;
}


vec2 compute_envelope_grid_center() {
    vec2 projected_bottom_left  = project_envelope(vec3(0.0, 0.0, 0.0));
    vec2 projected_bottom_right = project_envelope(vec3(float(DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS - 1), 0.0, 0.0));
    vec2 projected_top_left     = project_envelope(vec3(0.0, 1.0, float(NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS - 1)));
    vec2 projected_top_right    = project_envelope(vec3(float(DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS - 1), 1.0, float(NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS - 1)));

    vec2 minimum_corner = min(min(projected_bottom_left, projected_bottom_right),
                              min(projected_top_left, projected_top_right));
    vec2 maximum_corner = max(max(projected_bottom_left, projected_bottom_right),
                              max(projected_top_left, projected_top_right));
    return (minimum_corner + maximum_corner) * 0.5;
}


float distance_to_line(vec2 pixel_coordinate, vec2 line_start, vec2 line_end) {
    vec2 line_vector = line_end - line_start;
    float line_length_squared = dot(line_vector, line_vector);
    float projection_factor = dot(pixel_coordinate - line_start, line_vector) / line_length_squared;
    float clamped_projection_factor = clamp(projection_factor, 0.0, 1.0);
    vec2 closest_point = line_start + clamped_projection_factor * line_vector;
    return distance(pixel_coordinate, closest_point);
}
