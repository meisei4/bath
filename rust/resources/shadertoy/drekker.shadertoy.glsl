
const float NORMAL_CENTER_OFFSET = 0.5;

#define JITTERED_SUPERSAMPLE
const int   JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL = 64;
const float JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F
    = float(JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL);
const vec2  JITTERED_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET = vec2(NORMAL_CENTER_OFFSET);
const float JITTERED_RESONATE_STEP_X                         = 0.754877669;
const float JITTERED_RESONATE_STEP_Y                         = 0.569840296;
const vec2  JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR        = vec2(JITTERED_RESONATE_STEP_X, JITTERED_RESONATE_STEP_Y);

vec2 position_jitter_subpixel(vec2 current_jitter_subpixel_position) {
    vec2 next_jitter_subpixel_position = current_jitter_subpixel_position + JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR;
    vec2 next_jitter_subpixel_position_fraction = fract(next_jitter_subpixel_position);
    vec2 centered_next_jitter_subpixel_position = next_jitter_subpixel_position_fraction - NORMAL_CENTER_OFFSET;
    return centered_next_jitter_subpixel_position;
}

const float TEXTURE_LOD_BASE_LEVEL                    = 0.0;
const float GRID_COLUMNS                              = 11.0;
const float GRID_ROWS                                 = 7.0;
const float LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY = 0.10;
const float LOCAL_CELL_SPACE_BOUNDARY_LEFT            = 0.10;
const float LOCAL_CELL_SPACE_BOUNDARY_RIGHT           = 0.90;
const float LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR    = 0.10;
const float HYPERBOLIC_AMPLITUDE                      = 0.01;
const float THICKNESS_AMPLITUDE                       = 200.0;

#define ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(color, x, y)                                                        \
    {                                                                                                                  \
        if (abs(y) > LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY || (x) < LOCAL_CELL_SPACE_BOUNDARY_LEFT                 \
            || (x) > LOCAL_CELL_SPACE_BOUNDARY_RIGHT) {                                                                \
            (color).a = 0.0;                                                                                           \
        }                                                                                                              \
    }

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

float compute_horizontal_hyperbolic_curvature(float cell_progress_x) {
    float left_curve_downwards = HYPERBOLIC_AMPLITUDE * ((1.0 / cell_progress_x) - 1.0);
    float right_curve_upwards  = HYPERBOLIC_AMPLITUDE * ((1.0 / (1.0 - cell_progress_x)) - 1.0);
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

float compute_vertical_cubic_thickness(float local_cell_progress_y) {
    return local_cell_progress_y * local_cell_progress_y * local_cell_progress_y;
}

vec4 sample_uv_in_grid_space(vec2 uv, vec2 grid_dimensions) {
    float column_index                    = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float local_cell_progress_x           = compute_local_cell_progress_x(uv.x, grid_dimensions.x);
    float horizontal_hyperbolic_curvature = compute_horizontal_hyperbolic_curvature(local_cell_progress_x);

    float vertical_row_shift_per_column = LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR * column_index;
    float row_index                     = compute_discrete_row_index(
        uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);
    float local_cell_progress_y = compute_local_cell_progress_y(
        uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);

    float phase_shifted_local_cell_progress_y = local_cell_progress_y - NORMAL_CENTER_OFFSET;
    float vertical_cubic_thickness            = compute_vertical_cubic_thickness(phase_shifted_local_cell_progress_y);
    float vertical_cubic_thickness_normal     = vertical_cubic_thickness / iResolution.y;
    float amplified_vertical_thickness        = THICKNESS_AMPLITUDE * vertical_cubic_thickness_normal;

    float column_normal = column_index / grid_dimensions.x;
    float row_normal    = row_index / grid_dimensions.y;
    float u             = column_normal;
    float v             = row_normal + amplified_vertical_thickness - vertical_row_shift_per_column;

    vec4 src_color = textureLod(iChannel0, vec2(u, v), TEXTURE_LOD_BASE_LEVEL);
    ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(src_color, local_cell_progress_x, local_cell_progress_y);
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

#define BORDER_CLEARING
void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv              = frag_coord / iResolution.y;
    vec2 grid_dimensions = compute_grid_dimensions();
    vec4 src_color       = sample_uv_in_grid_space(uv, grid_dimensions);
#ifdef BORDER_CLEARING
    if (src_color.a > 0.0) {
        src_color.a = 1.0;
        frag_color  = src_color;
    } else
#endif
    {
        src_color   = jitter_supersample(frag_coord, grid_dimensions);
        src_color.a = 1.0;
        frag_color  = src_color;
    }
}
