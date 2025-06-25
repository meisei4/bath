#define TEXTURE_LOD_BASE_LEVEL 0.0
#define GRID_COLUMNS 9.125 / 2.33
#define GRID_ROWS 7.0 / 3.0
#define MARGIN 0.1
#define LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY MARGIN
#define LOCAL_CELL_SPACE_BOUNDARY_LEFT MARGIN
#define LOCAL_CELL_SPACE_BOUNDARY_RIGHT 1.0 - MARGIN

#define ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(color, x, y)                                                        \
    {                                                                                                                  \
        if (abs(y) > LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY || (x) < LOCAL_CELL_SPACE_BOUNDARY_LEFT                 \
            || (x) > LOCAL_CELL_SPACE_BOUNDARY_RIGHT) {                                                                \
            (color).a = 0.0;                                                                                           \
        }                                                                                                              \
    }

#define NORMAL_CENTER_OFFSET 0.5
#define JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL 64
#define JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F                                                   \
    float(JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL)
#define JITTERED_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET vec2(NORMAL_CENTER_OFFSET)
#define JITTERED_RESONATE_STEP_X 0.754877669
#define JITTERED_RESONATE_STEP_Y 0.569840296
#define JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR vec2(JITTERED_RESONATE_STEP_X, JITTERED_RESONATE_STEP_Y)

vec2 position_jitter_subpixel(vec2 current_jitter_subpixel_position) {
    vec2 next_jitter_subpixel_position = current_jitter_subpixel_position + JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR;
    vec2 next_jitter_subpixel_position_fraction = fract(next_jitter_subpixel_position);
    vec2 centered_next_jitter_subpixel_position = next_jitter_subpixel_position_fraction - NORMAL_CENTER_OFFSET;
    return centered_next_jitter_subpixel_position;
}
