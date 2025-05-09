//this is a verbose study of cotterrzz max drecker reproduction https://www.shadertoy.com/view/33SSDm
shader_type canvas_item;
//render_mode blend_disabled; THIS WILL MAYBE MAKE THE ALPHA CHANNEL 1.0 assignment not needed/optimized???

//TODO: MAY 9th next time explore more with macros to separate out each of the transformations performed to truly
// be able to visualize what each operation is in charge of 
uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;

const float TEXTURE_LOD_BASE_LEVEL = 0.0;

const float GRID_COLUMNS          = 11.0;
const float GRID_ROWS             = 7.0;

const float LOCAL_CELL_SPACE_UPPER_AND_LOWER_PADDING = 0.10;
const float LOCAL_CELL_SPACE_MARGIN_LEFT  = 0.10; // 10% margin on the left
const float LOCAL_CELL_SPACE_MARGIN_RIGHT = 0.90; // remember this is from x origin, to achieve 10% margin on the right
const float LOCAL_CELL_SPACE_CENTER_OFFSET         = 0.5; // center of a grid cell
const float LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR = 0.10; //shift vertically by 10% of the grid's row height

//TODO: study this more
const float HYPERBOLIC_AMPLITUDE                = 0.01;
//TODO: figure out how to perhaps integrate a more intuitive scalar that beefs up the vertical thickness effect
const float THICKNESS_AMPLITUDE                 = 100.0;

//TODO: haha
const int   SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE = 64;
const float SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE_F = float(SUPERSAMPLE_ANTI_ALIASING_PIXELS_PER_SAMPLE);
const vec2  WEYL_SEQUENCE_JITTER_INITIAL_STATE = vec2(LOCAL_CELL_SPACE_CENTER_OFFSET);
const vec2  WEYL_SEQUENCE_STEP                = vec2(0.754877669, 0.569840296);


#define CLEAR_CELL_BORDERS_ALPHA(color, x, y) {                   \
            if ( abs(y) > LOCAL_CELL_SPACE_UPPER_AND_LOWER_PADDING \
               || (x)   < LOCAL_CELL_SPACE_MARGIN_LEFT            \
               || (x)   > LOCAL_CELL_SPACE_MARGIN_RIGHT) {       \
                (color).a = 0.0;                               \
            }                                                \
        }

vec2 compute_grid_dimensions() {
    return vec2(GRID_COLUMNS, GRID_ROWS);
}

float compute_discrete_column_index(float uv_x, float grid_width_in_columns) {
    float column_units_x = uv_x * grid_width_in_columns;
    float discrete_column_index = floor(column_units_x);
    return discrete_column_index;
}

float compute_local_cell_progress_x(float uv_x, float grid_width_in_columns) {
    float column_units_x    = uv_x * grid_width_in_columns;
    float local_cell_progress_x = fract(column_units_x);
    return local_cell_progress_x;
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
float compute_horizontal_hyperbolic_curvature(float cell_progress_x) {
    float left_curve_downwards    = HYPERBOLIC_AMPLITUDE * ((1.0 / cell_progress_x) - 1.0);
    float right_curve_upwards     = HYPERBOLIC_AMPLITUDE * ((1.0 / (1.0 - cell_progress_x)) - 1.0);
    float curvature               = left_curve_downwards - right_curve_upwards;
    return curvature;
}

float compute_discrete_row_index(float uv_y, float vertical_row_shift_per_column, float grid_height_in_rows, float horizontal_hyperbolic_curvature) {
    float shifted_uv_y               = uv_y + vertical_row_shift_per_column;
    float shifted_row_units_y        = shifted_uv_y * grid_height_in_rows;
    float curved_shifted_row_units_y = shifted_row_units_y + horizontal_hyperbolic_curvature;
    float discrete_row_index_after_shift_and_curve = floor(curved_shifted_row_units_y);
    return discrete_row_index_after_shift_and_curve;
}

float compute_local_cell_progress_y(float uv_y, float vertical_row_shift_per_column, float grid_height_in_rows, float horizontal_hyperbolic_curvature) {
    float shifted_uv_y               = uv_y + vertical_row_shift_per_column;
    float shifted_row_units_y        = shifted_uv_y * grid_height_in_rows;
    float curved_shifted_row_units_y = shifted_row_units_y + horizontal_hyperbolic_curvature;
    float local_cell_progress_y_after_shift_and_curve = fract(curved_shifted_row_units_y);
    return local_cell_progress_y_after_shift_and_curve;
}

/*
Phase shifted local cell space vertical CUBIC thickening 
thickest between cells, thinnest in the middle of cells
  f(y)=y³
                    ^ +0.125            *
                    |                  *
                    |                 *
                    |                *
                    |               *
                    |             **
                    |           **
                    | 0    ****
–0.5 ----------------+------------------ +0.5
           ****     |
       **           |
     **             |
    *               |
   *                |
  *                 |
 *                  |
*                   v –0.125

CUBIC THICKENING NUMERIC INTUTION:
Local cell Y is offset to achieve the thickness phasing between each cell VERTICALLY
   y       y³       f(y) absolute thickness      normalized to max (0.125)
 -----   ---------   -------------------------     ------------------------
–0.50    –0.125       0.125                          1.00
–0.25    –0.015625    0.015625                       0.125
–0.10    –0.001       0.001                          0.008
–0.02    –0.000008    0.000008                       0.000064
 0.00     0.00        0.00                           0.00
+0.02     0.000008    0.000008                       0.000064
+0.10     0.001       0.001                          0.008
+0.25     0.015625    0.015625                       0.125
+0.50     0.125       0.125                          1.00
*/

float compute_vertical_cubic_thickness(float local_cell_progress_y) {
    return local_cell_progress_y * local_cell_progress_y * local_cell_progress_y;
}

vec4 sample_uv_in_grid_space(vec2 uv, vec2 grid_dimensions) {
    float column_index                   = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float local_cell_progress_x           = compute_local_cell_progress_x(uv.x, grid_dimensions.x);
    float horizontal_hyperbolic_curvature = compute_horizontal_hyperbolic_curvature(local_cell_progress_x);

    float vertical_row_shift_per_column    = LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR * column_index;
    float row_index                       = compute_discrete_row_index(uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);
    float local_cell_progress_y            = compute_local_cell_progress_y(uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);

    float phase_shifted_local_cell_progress_y = local_cell_progress_y - LOCAL_CELL_SPACE_CENTER_OFFSET;
    float vertical_cubic_thickness            = compute_vertical_cubic_thickness(phase_shifted_local_cell_progress_y);
    float vertical_cubic_thickness_normal     = vertical_cubic_thickness / iResolution.y;
    float amplified_vertical_thickness        = THICKNESS_AMPLITUDE * vertical_cubic_thickness_normal;

    float column_normal = column_index / grid_dimensions.x;
    float row_normal    = row_index    / grid_dimensions.y;
    float u = column_normal;
    float v = row_normal + amplified_vertical_thickness - vertical_row_shift_per_column;

    vec4 src_color = textureLod(iChannel0, vec2(u,v), TEXTURE_LOD_BASE_LEVEL);

    CLEAR_CELL_BORDERS_ALPHA(src_color, local_cell_progress_x, local_cell_progress_y);

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
        vec2 jitter_offset     = current_sequence_state - LOCAL_CELL_SPACE_CENTER_OFFSET;
        vec2 sample_coord      = frag_coord + jitter_offset;
        vec2 uv                = sample_coord / iResolution.y;
        weyl_sequence_jitter   = weyl_sequence_jitter + sample_uv_in_grid_space(uv, grid_dimensions);
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
    vec4 src_color = sample_uv_in_grid_space(uv, grid_dimensions);
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
