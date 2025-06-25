#version 330
#include "supersampling.glsl"

uniform sampler2D iChannel1;
uniform vec2      iResolution;

#define LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR 0.0125
#define HYPERBOLIC_AMPLITUDE 0.008

float hyper(float column_index) {
    float hyper = HYPERBOLIC_AMPLITUDE;
    return hyper;
}

#define CUBIC_SQUISH_SCALAR 90.0
#define NORMAL_OFFSET_MAIN 0.5

vec2 compute_grid_dimensions() { return vec2(GRID_COLUMNS, GRID_ROWS); }

float compute_discrete_column_index(float uv_x, float grid_width_in_columns) {
    float column_units_x        = uv_x * grid_width_in_columns;
    float discrete_column_index = floor(column_units_x);
    return discrete_column_index;
}

float compute_local_cell_progress_x(float uv_x, float grid_width_in_columns) {
    float column_units_x        = uv_x * grid_width_in_columns;
    float local_cell_progress_x = fract(column_units_x);
    return local_cell_progress_x;
}

float compute_horizontal_hyperbolic_curvature(float hyperbolic_amplitude, float cell_progress_x) {
    float left_curve_downwards = hyperbolic_amplitude * ((1.0 / cell_progress_x) - 1.0);
    float right_curve_upwards  = hyperbolic_amplitude * ((1.0 / (1.0 - cell_progress_x)) - 1.0);
    float curvature            = left_curve_downwards - right_curve_upwards;
    return curvature;
}

float compute_discrete_row_index(
    float uv_y, float vertical_row_shift_per_column, float grid_height_in_rows, float horizontal_hyperbolic_curvature) {
    float shifted_uv_y                             = uv_y + vertical_row_shift_per_column;
    float shifted_row_units_y                      = shifted_uv_y * grid_height_in_rows;
    float curved_shifted_row_units_y               = shifted_row_units_y + horizontal_hyperbolic_curvature;
    float discrete_row_index_after_shift_and_curve = floor(curved_shifted_row_units_y);
    return discrete_row_index_after_shift_and_curve;
}

float compute_local_cell_progress_y(
    float uv_y, float vertical_row_shift_per_column, float grid_height_in_rows, float horizontal_hyperbolic_curvature) {
    float shifted_uv_y                                = uv_y + vertical_row_shift_per_column;
    float shifted_row_units_y                         = shifted_uv_y * grid_height_in_rows;
    float curved_shifted_row_units_y                  = shifted_row_units_y + horizontal_hyperbolic_curvature;
    float local_cell_progress_y_after_shift_and_curve = fract(curved_shifted_row_units_y);
    return local_cell_progress_y_after_shift_and_curve;
}

float compute_vertical_cubic_color_squish(float local_cell_progress_y) {
    return local_cell_progress_y * local_cell_progress_y * local_cell_progress_y;
}

vec4 sample_uv_in_grid_space(vec2 uv, vec2 grid_dimensions) {
    float column_index          = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float local_cell_progress_x = compute_local_cell_progress_x(uv.x, grid_dimensions.x);
    float horizontal_hyperbolic_curvature
        = compute_horizontal_hyperbolic_curvature(hyper(column_index), local_cell_progress_x);

    float vertical_row_shift_per_column = LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR * column_index;
    float row_index                     = compute_discrete_row_index(
        uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);
    float local_cell_progress_y = compute_local_cell_progress_y(
        uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);
    float phase_shifted_local_cell_progress_y = local_cell_progress_y - NORMAL_CENTER_OFFSET;
    float cubic_color_squish_cell_space       = compute_vertical_cubic_color_squish(local_cell_progress_y);
    float cubic_color_squish_normal           = cubic_color_squish_cell_space / iResolution.y;
    float cubic_color_squish_scaled           = CUBIC_SQUISH_SCALAR * cubic_color_squish_normal;

    float column_normal = column_index / grid_dimensions.x;
    float row_normal    = row_index / grid_dimensions.y;
    float u             = column_normal;
    float v             = row_normal + cubic_color_squish_scaled - vertical_row_shift_per_column;
    vec4  texture_color = texture(iChannel1, vec2(u, v));

    ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(texture_color, local_cell_progress_x, local_cell_progress_y);

    vec4 src_color = texture_color;
    return src_color;
}

vec4 jitter_supersample(vec2 frag_coord, vec2 grid_dimensions) {
    vec4 jittered_distribution              = vec4(0.0);
    int  remaining_subpixels_in_supersample = JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL;
    vec2 current_jitter_subpixel_position   = JITTERED_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET;
    while (remaining_subpixels_in_supersample > 0) {
        current_jitter_subpixel_position   = position_jitter_subpixel(current_jitter_subpixel_position);
        vec2 sample_coord                  = frag_coord + current_jitter_subpixel_position;
        vec2 uv                            = sample_coord / iResolution.y;
        jittered_distribution              = jittered_distribution + sample_uv_in_grid_space(uv, grid_dimensions);
        remaining_subpixels_in_supersample = remaining_subpixels_in_supersample - 1;
    }
    return jittered_distribution / JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F;
}

#define SUPERSAMPLE(frag_coord, grid_dimensions) jitter_supersample(frag_coord, grid_dimensions)

in vec2  fragTexCoord;
in vec4  fragColor;
out vec4 finalColor;
void     main() {
    vec2 frag_coord      = vec2(fragTexCoord.x * iResolution.x, fragTexCoord.y * iResolution.y);
    vec2 uv              = frag_coord / iResolution.y;
    vec2 grid_dimensions = compute_grid_dimensions();
    vec4 src_color       = sample_uv_in_grid_space(uv, grid_dimensions);
    if (src_color.a > 0.0) {
        src_color.a = 1.0;
        finalColor  = src_color;
    } else {
        src_color   = SUPERSAMPLE(frag_coord, grid_dimensions);
        src_color.a = 1.0;
        finalColor  = src_color;
    }
}
