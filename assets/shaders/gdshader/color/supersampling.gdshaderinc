#define TEXTURE_LOD_BASE_LEVEL 0.0
//#define GRID_COLUMNS           1.125
//#define GRID_ROWS              2.74
#define GRID_COLUMNS             9.125 / 2.33
#define GRID_ROWS                7.0 / 3.0

#define MARGIN 0.1 //TODO: idk

#define LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY MARGIN
#define LOCAL_CELL_SPACE_BOUNDARY_LEFT            MARGIN// 10% margin on the left
#define LOCAL_CELL_SPACE_BOUNDARY_RIGHT           1.0 - MARGIN // remember this is from x origin, to achieve 10% margin on the right subtract from 1

#define ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(color, x, y) {  \
            if ( abs(y) > LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY \
               || (x)   < LOCAL_CELL_SPACE_BOUNDARY_LEFT            \
               || (x)   > LOCAL_CELL_SPACE_BOUNDARY_RIGHT) {       \
                (color).a = 0.0;                               \
            }                                                \
        }
                //discard; \
                //(color).a = 1.0;                               \
                //(color).r = 1.0;                               \
                //(color).g = 0.0;                               \
                //(color).b = 0.0;                               \

#define NORMAL_CENTER_OFFSET   0.5 // center of any normal space
#define UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL   64
#define UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F float(UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL)

#define UNIFORM_RESONATE_STEP                                       1.0 / sqrt(UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F)
#define UNIFORM_RESONATE_STEP_FRACTIONAL                            1.0 / UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F
#define UNIFORM_SUPERSAMPLE_RESONATE_STEP_VECTOR_FRACTIONAL         vec2(UNIFORM_RESONATE_STEP, UNIFORM_RESONATE_STEP_FRACTIONAL)
#define UNIFORM_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET_FRACTIONAL  NORMAL_CENTER_OFFSET * UNIFORM_SUPERSAMPLE_RESONATE_STEP_VECTOR_FRACTIONAL

//#define INITIAL_UNIFORM_STEP                                      vec2(NORMAL_CENTER_OFFSET)
#define INITIAL_UNIFORM_STEP                                        UNIFORM_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET_FRACTIONAL

#define UNIFORM_STEP(current_uniform_subpixel_position)             position_uniform_subpixel_fractional_stepping(current_uniform_subpixel_position)
//#define UNIFORM_STEP(current_uniform_subpixel_position)           position_uniform_subpixel_normal_stepping(current_uniform_subpixel_position)

#define JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL   64
#define JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F float(JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL)
#define JITTERED_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET             vec2(NORMAL_CENTER_OFFSET)
#define JITTERED_RESONATE_STEP_X                                     0.754877669
#define JITTERED_RESONATE_STEP_Y                                     0.569840296
#define JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR                    vec2(JITTERED_RESONATE_STEP_X, JITTERED_RESONATE_STEP_Y)
// POTENTIAL OTHER RESONATE STEP VALUES FOR JITTER (COMMON "IRRATIONAL" NUMBERS OR OTHERWISE NUMBERS NON-COMMENSURATE TO 1)
// φ−1  = 0.6180339887 (the golden ratio minus one?)
// √2−1 = 0.4142135623
// √3−1 = 0.7320508076

vec2 position_uniform_subpixel_fractional_stepping(vec2 current_uniform_subpixel_position) {
    vec2 next_uniform_subpixel_position          = current_uniform_subpixel_position + UNIFORM_SUPERSAMPLE_RESONATE_STEP_VECTOR_FRACTIONAL;
    vec2 next_uniform_subpixel_position_fraction = fract(next_uniform_subpixel_position);
    vec2 centered_next_uniform_subpixel_position = next_uniform_subpixel_position_fraction - NORMAL_CENTER_OFFSET;
    return centered_next_uniform_subpixel_position;
}

vec2 position_uniform_subpixel_normal_stepping(vec2 current_uniform_subpixel_position) {
    current_uniform_subpixel_position.x         = current_uniform_subpixel_position.x + UNIFORM_RESONATE_STEP;
    if (current_uniform_subpixel_position.x >= 1.0) {
        current_uniform_subpixel_position.x     = current_uniform_subpixel_position.x - 1.0;
        current_uniform_subpixel_position.y     = current_uniform_subpixel_position.y + UNIFORM_RESONATE_STEP;
        if (current_uniform_subpixel_position.y >= 1.0) {
            current_uniform_subpixel_position.y = current_uniform_subpixel_position.y - 1.0;
        }
    }
    return current_uniform_subpixel_position;
}

vec2 position_jitter_subpixel(vec2 current_jitter_subpixel_position) {
    vec2 next_jitter_subpixel_position          = current_jitter_subpixel_position + JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR;
    vec2 next_jitter_subpixel_position_fraction = fract(next_jitter_subpixel_position);
    vec2 centered_next_jitter_subpixel_position = next_jitter_subpixel_position_fraction - NORMAL_CENTER_OFFSET;
    return centered_next_jitter_subpixel_position;
}
