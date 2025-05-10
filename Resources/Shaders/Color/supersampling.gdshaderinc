const float NORMAL_CENTER_OFFSET   = 0.5; // center of any normal space


#define UNIFORM_SUPERSAMPLE
    //TODO: test different super sample resolutions
    const int   UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL   = 64;
    const float UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F = float(UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL);
    const float UNIFORM_RESONATE_STEP                                       = 1.0 / sqrt(UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F);
    const vec2  UNIFORM_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET_ROW_MAJOR   = vec2(NORMAL_CENTER_OFFSET);
    #define DISTRIBUTE_WITH_FRACTIONAL_STEP
        const float UNIFORM_RESONATE_STEP_FRACTIONAL                           = 1.0 / UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F;
        const vec2  UNIFORM_SUPERSAMPLE_RESONATE_STEP_VECTOR_FRACTIONAL        = vec2(UNIFORM_RESONATE_STEP, UNIFORM_RESONATE_STEP_FRACTIONAL);
        const vec2  UNIFORM_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET_FRACTIONAL = NORMAL_CENTER_OFFSET * UNIFORM_SUPERSAMPLE_RESONATE_STEP_VECTOR_FRACTIONAL;


//#define JITTERED_SUPERSAMPLE
    const int   JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL   = 64;
    const float JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F = float(JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL);
    const vec2  JITTERED_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET             = vec2(NORMAL_CENTER_OFFSET);
    const float JITTERED_RESONATE_STEP_X                                     = 0.754877669;
    const float JITTERED_RESONATE_STEP_Y                                     = 0.569840296;
    const vec2  JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR                    = vec2(JITTERED_RESONATE_STEP_X, JITTERED_RESONATE_STEP_Y);
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
    // .yx???????
    //vec2 next_jitter_subpixel_position          = current_jitter_subpixel_position + JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR.yx;
    vec2 next_jitter_subpixel_position          = current_jitter_subpixel_position + JITTERED_SUPERSAMPLE_RESONATE_STEP_VECTOR;
    vec2 next_jitter_subpixel_position_fraction = fract(next_jitter_subpixel_position);
    vec2 centered_next_jitter_subpixel_position = next_jitter_subpixel_position_fraction - NORMAL_CENTER_OFFSET;
    return centered_next_jitter_subpixel_position;
}
