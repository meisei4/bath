shader_type canvas_item;
//render_mode blend_disabled; THIS WILL MAYBE MAKE THE ALPHA CHANNEL 1.0 assignment not needed/optimized???

//this is a verbose study of cotterrzz max drecker reproduction https://www.shadertoy.com/view/33SSDm
uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;

const int   SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE = 64;
const float SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE_F = float(SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE);
const vec2  WEYL_SEQUENCE_STEP                = vec2(0.754877669, 0.569840296);
const float GRID_CELL_CENTER_OFFSET           = 0.5;
const vec2  WEYL_SEQUENCE_JITTER_INITIAL_STATE = vec2(GRID_CELL_CENTER_OFFSET);

const float TEXTURE_LOD_BASE_LEVEL        = 0.0;
const float DEFAULT_GRID_COLUMNS          = 11.0;
const float DEFAULT_GRID_ROWS             = 7.0;
const float GRID_CELLS_OUTER_PADDING        = 0.10;
// THE MARGINS ARE ALL COMPUTED FROM x= 0.0 origin, so to get the
// LEFT side of the cell you just take x origin + 0.10
const float GRID_CELLS_LATERAL_MARGIN_LEFT  = 0.10;
// RIGHT side of the cell you just take x origin + 1.0 - 0.10 = 0.90
const float GRID_CELLS_LATERAL_MARGIN_RIGHT = 0.90;

const float GRID_COLUMNS_VERTICAL_SHIFT_SCALAR = 0.10;
const float HYPERBOLIC_AMPLITUDE               = 0.01;
const float THICKNESS_FALLOFF_SCALE            = 100.0;


#define CLEAR_CELL_BORDERS_ALPHA(color, x, y) {              \
            if ( abs(y) > GRID_CELLS_OUTER_PADDING           \
               || (x)   < GRID_CELLS_LATERAL_MARGIN_LEFT     \
               || (x)   > GRID_CELLS_LATERAL_MARGIN_RIGHT) { \
                (color).a = 0.0;                            \
            }                                              \
        }

vec2 compute_grid_dimensions() {
    return vec2(DEFAULT_GRID_COLUMNS, DEFAULT_GRID_ROWS);
}

float compute_discrete_column_index(float uv_x, float column_count) {
    float column_units_x = uv_x * column_count;
    float discrete_column_index = floor(column_units_x);
    return discrete_column_index;
}

float compute_cell_progress_x(float uv_x, float column_count) {
    float column_units_x    = uv_x * column_count;
    float inner_cell_progress_x = fract(column_units_x);
    return inner_cell_progress_x;
}

/// TODO: THE BELOW MATH IS ALL WRONG ITS APPROACHING 1 vs apporaching 0,
/// and i messed it up in ascii, fix it later, the texture gets flipped in
/// the viewport so i messed up the orientation and that messed up my understanding of the limits...
/*
                                               . ^ INFINITY +∞
                                                 |
                                               ' |
                                              .  |
                                             /   |
                                           .     |
                                        .        |
                                 . . .         0 |
        <---------------.-.--*-'-'----------------+
        | 1      . . .
        |     .
        |   .
        |  /
        | .
        |'
        |. -INFINITY -∞
        v

Let k (kappa?) = HYPERBOLIC_AMPLITUDE
Definition of `compute_vertical_curvature_hyperbolic()`:
    f(x) = left_curve_downwards – right_curve_upwards
         = (k/x – k) – (k/(1–x) – k)
         = k · (1/x – 1/(1–x))

          k · (1 – 2x)
         = ———————————
           x · (1 – x)

BEHAVIOR/BOUNDS/LIMITS:
    x -> 0:
            k · (1)       k
    f(x) =  ————————  =  ————  -> +∞ (+INFINITY)
             x · (1)      x

    x == 0.5:
      numerator = k · (1 – 2 · 0.5) = 0
            -> f(0.5) = 0

    x -> 1⁻:
      f(x) =  k · (1 – 2 · 1)      -k · (1)       –k
             —————————————————  =  —————————  =  ——————  -> -∞ (-INFINITY)
                1 · (1 – x)         1 – x        1 – x
*/
float compute_hyperbolic_curvature(float cell_progress_x) {
    float left_curve_downwards    = (HYPERBOLIC_AMPLITUDE / cell_progress_x) - HYPERBOLIC_AMPLITUDE;
    float inverse_cell_progress_x = 1.0 - cell_progress_x;
    float right_curve_upwards     = (HYPERBOLIC_AMPLITUDE / inverse_cell_progress_x) - HYPERBOLIC_AMPLITUDE;
    float curvature               = left_curve_downwards - right_curve_upwards;
    return curvature;
}

float compute_discrete_row_index(float uv_y, float column_index, vec2 grid_dimensions, float curvature) {
    float shifted_uv_y               = uv_y + column_index * GRID_COLUMNS_VERTICAL_SHIFT_SCALAR;
    float shifted_row_units_y        = shifted_uv_y * grid_dimensions.y;
    float curved_shifted_row_units_y = shifted_row_units_y + curvature;
    float discrete_row_index_after_shift_and_curve = floor(curved_shifted_row_units_y);
    return discrete_row_index_after_shift_and_curve;
}

float compute_cell_progress_y(float uv_y,float column_index, vec2 grid_dimensions,float curvature) {
    float shifted_uv_y               = uv_y + column_index * GRID_COLUMNS_VERTICAL_SHIFT_SCALAR;
    float shifted_row_units_y        = shifted_uv_y * grid_dimensions.y;
    float curved_shifted_row_units_y = shifted_row_units_y + curvature;
    //TODO: why is this offset by center of cell?
    float half_cell_progress_y_after_shift_and_curve = fract(curved_shifted_row_units_y) - GRID_CELL_CENTER_OFFSET;
    return half_cell_progress_y_after_shift_and_curve;
}

// TODO: start here tomorrow i have no idea whats happening, and how this results in the fat middle of the cell
// idk how it combines with the hyperbola in the fattening in the middle...
float compute_cubic_thickness_offset(float cell_progress_y) {
    return cell_progress_y * cell_progress_y * cell_progress_y;
}

vec4 sample_grid_at_fragment(vec2 uv, vec2 grid_dimensions) {
    float column_index           = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float cell_progress_x        = compute_cell_progress_x(uv.x, grid_dimensions.x);
    float vertical_curvature     = compute_hyperbolic_curvature(cell_progress_x);
    float row_index              = compute_discrete_row_index(uv.y, column_index, grid_dimensions, vertical_curvature);
    float cell_progress_y        = compute_cell_progress_y(uv.y, column_index, grid_dimensions, vertical_curvature);
    float cubic_thickness_offset = compute_cubic_thickness_offset(cell_progress_y);

    float column_normal            = column_index / grid_dimensions.x;
    float row_normal               = row_index / grid_dimensions.y;
    float vertical_thickness        = cubic_thickness_offset / iResolution.y;
    float scaled_vertical_thickness = vertical_thickness * THICKNESS_FALLOFF_SCALE;
    float vertical_shift_per_column = column_index * GRID_COLUMNS_VERTICAL_SHIFT_SCALAR;

    float u = column_normal;
    float v = row_normal + scaled_vertical_thickness - vertical_shift_per_column;

    vec4 src_color = textureLod(iChannel0, vec2(u,v), TEXTURE_LOD_BASE_LEVEL);

    CLEAR_CELL_BORDERS_ALPHA(src_color, cell_progress_x, cell_progress_y);

    return src_color;
}

vec2 advance_weyl_sequence(vec2 current_sequence_state) {
    vec2 next_sequence_state          = current_sequence_state + WEYL_SEQUENCE_STEP.yx;
    vec2 next_sequence_state_fraction = fract(next_sequence_state);
    return next_sequence_state_fraction;
}

vec4 super_sample_with_weyl_sequence_jitter(vec2 frag_coord, vec2 grid_dimensions) {
    vec4 weyl_sequence_jitter   = vec4(0.0);
    int  remaining_pixels_in_super_sample = SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE;
    vec2 current_sequence_state = WEYL_SEQUENCE_JITTER_INITIAL_STATE;
    while (remaining_pixels_in_super_sample > 0) {
        current_sequence_state = advance_weyl_sequence(current_sequence_state);
        //TODO: why is this offset by center of cell?
        vec2 jitter_offset     = current_sequence_state - GRID_CELL_CENTER_OFFSET;
        vec2 sample_coord      = frag_coord + jitter_offset;
        vec2 uv                = sample_coord / iResolution.y;
        weyl_sequence_jitter   = weyl_sequence_jitter + sample_grid_at_fragment(uv, grid_dimensions);
        remaining_pixels_in_super_sample = remaining_pixels_in_super_sample - 1;
    }
    return weyl_sequence_jitter;
}

#define BORDER_CLEARING
void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 frag_coord     = vec2(UV.x * iResolution.x,
                              UV.y * iResolution.y);
    vec2 uv             = frag_coord / iResolution.y;
    //vec2 uv           = frag_coord / iResolution.y;
    vec2 grid_dimensions = compute_grid_dimensions();
    vec4 src_color = sample_grid_at_fragment(uv, grid_dimensions);
    #ifdef BORDER_CLEARING
    if (src_color.a > 0.0) {
        src_color.a = 1.0; // HARDCODE STRAIGHT ALPHA BLEND TO NOT OCCUR (i.e. destination.rgb * 0.0)
        COLOR = src_color;
        //frag_color = src_color;
    } else
    #endif //HACK...
    {
        src_color = super_sample_with_weyl_sequence_jitter(frag_coord, grid_dimensions);
        src_color = src_color / SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE_F;
        src_color.a = 1.0; // HARDCODE STRAIGHT ALPHA BLEND TO NOT OCCUR (i.e. destination.rgb * 0.0)
        COLOR = src_color;
        //frag_color = src_color;
    }
}
