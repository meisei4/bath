//this is a verbose study of cotterrzz max drecker reproduction https://www.shadertoy.com/view/33SSDm
shader_type canvas_item;
//render_mode blend_disabled; THIS WILL MAYBE MAKE THE ALPHA CHANNEL 1.0 assignment not needed/optimized???
#include "res://Resources/Shaders/Color/supersampling.gdshaderinc"

uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;

const float TEXTURE_LOD_BASE_LEVEL = 0.0;

const float GRID_COLUMNS           = 11.0;
const float GRID_ROWS              = 7.0;

const float LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY = 0.10;
const float LOCAL_CELL_SPACE_BOUNDARY_LEFT            = 0.10; // 10% margin on the left
const float LOCAL_CELL_SPACE_BOUNDARY_RIGHT           = 0.90; // remember this is from x origin, to achieve 10% margin on the right
const float LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR    = 0.10; //shift vertically by 10% of the grid's row height

//TODO: try different amplitudes and perhaps figure out different curvature functions to work with
const float HYPERBOLIC_AMPLITUDE = 0.01;
const float THICKNESS_AMPLITUDE  = 200.0;

#define ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(color, x, y) {                   \
            if ( abs(y) > LOCAL_CELL_SPACE_UPPER_AND_LOWER_BOUNDARY \
               || (x)   < LOCAL_CELL_SPACE_BOUNDARY_LEFT            \
               || (x)   > LOCAL_CELL_SPACE_BOUNDARY_RIGHT) {       \
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

/// mirrored on accident, i messed it up in ascii, fix it later, the texture gets flipped in
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
        <---------------.-.--*-'-'---------------+
        | 1      . . .
        |     .
        |   .
        |  /
        | .
        |'
        |. -INFINITY -∞
        v

Let k (kappa?) = HYPERBOLIC_AMPLITUDE
Definition of compute_vertical_curvature_hyperbolic():
    f(x) = left_curve_downwards – right_curve_upwards
         = (k/x – k) – (k/(1–x) – k)
         = k · (1/x – 1/(1–x))

           k · (1 – 2x)
         = ------------
           x · (1 – x)

BEHAVIOR/BOUNDS/LIMITS:
    x -> 0:
             k · (1)        k
    f(x) =  ---------  =  -----  -> +∞ (+INFINITY)
             x · (1)        x

    x == 0.5:
      numerator = k · (1 – 2 · 0.5) = 0
            -> f(0.5) = 0

    x -> 1⁻:
      f(x) =  k · (1 – 2 · 1)      -k · (1)      –k
             ----------------- =  --------- =  -------  -> -∞ (-INFINITY)
                1 · (1 – x)         1 – x       1 – x
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
–0.5 ---------------+------------------ +0.5
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
    float column_index                    = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float local_cell_progress_x           = compute_local_cell_progress_x(uv.x, grid_dimensions.x);
    float horizontal_hyperbolic_curvature = compute_horizontal_hyperbolic_curvature(local_cell_progress_x);

    float vertical_row_shift_per_column    = LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR * column_index;
    float row_index                        = compute_discrete_row_index(uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);
    float local_cell_progress_y            = compute_local_cell_progress_y(uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);

    float phase_shifted_local_cell_progress_y = local_cell_progress_y - NORMAL_CENTER_OFFSET;
    float vertical_cubic_thickness            = compute_vertical_cubic_thickness(phase_shifted_local_cell_progress_y);
    float vertical_cubic_thickness_normal     = vertical_cubic_thickness / iResolution.y;
    float amplified_vertical_thickness        = THICKNESS_AMPLITUDE * vertical_cubic_thickness_normal;

    float column_normal = column_index / grid_dimensions.x;
    float row_normal    = row_index    / grid_dimensions.y;
    float u = column_normal;
    float v = row_normal + amplified_vertical_thickness - vertical_row_shift_per_column;

    vec4 src_color = textureLod(iChannel0, vec2(u,v), TEXTURE_LOD_BASE_LEVEL);
    ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(src_color, local_cell_progress_x, local_cell_progress_y);
    return src_color;
}


vec4 uniform_supersample(vec2 frag_coord, vec2 grid_dimensions) {
    vec4 uniform_distribution = vec4(0.0);
    int  remaining_subpixels_in_supersample     = UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL;
    #ifdef DISTRIBUTE_WITH_FRACTIONAL_STEP
        vec2  current_uniform_subpixel_position = UNIFORM_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET_FRACTIONAL;
    #else
        vec2  current_uniform_subpixel_position = UNIFORM_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET_ROW_MAJOR;
    #endif

    while (remaining_subpixels_in_supersample > 0) {
        #ifdef DISTRIBUTE_WITH_FRACTIONAL_STEP
            current_uniform_subpixel_position = position_uniform_subpixel_fractional_stepping(current_uniform_subpixel_position);
        #else
            current_uniform_subpixel_position = position_uniform_subpixel_normal_stepping(current_uniform_subpixel_position);
        #endif
        vec2 sample_coord                  = frag_coord + current_uniform_subpixel_position;
        vec2 uv                            = sample_coord / iResolution.y;
        uniform_distribution               = uniform_distribution + sample_uv_in_grid_space(uv, grid_dimensions);
        remaining_subpixels_in_supersample = remaining_subpixels_in_supersample - 1;
    }
    return uniform_distribution / UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F;
}

vec4 jitter_supersample(vec2 frag_coord, vec2 grid_dimensions) {
    vec4 jittered_distribution  = vec4(0.0);
    int  remaining_subpixels_in_supersample = JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL;
    vec2 current_jitter_subpixel_position   = JITTERED_SUPERSAMPLE_DISTRIBUTION_INITIAL_OFFSET;
    while (remaining_subpixels_in_supersample > 0) {
        current_jitter_subpixel_position    = position_jitter_subpixel(current_jitter_subpixel_position);
        vec2 sample_coord                   = frag_coord + current_jitter_subpixel_position;
        vec2 uv                             = sample_coord / iResolution.y;
        jittered_distribution               = jittered_distribution + sample_uv_in_grid_space(uv, grid_dimensions);
        remaining_subpixels_in_supersample  = remaining_subpixels_in_supersample - 1;
    }
    return jittered_distribution / JITTERED_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F;
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
        #ifdef UNIFORM_SUPERSAMPLE
            src_color = uniform_supersample(frag_coord, grid_dimensions);
        #endif
        #ifdef JITTERED_SUPERSAMPLE
            src_color = jitter_supersample(frag_coord, grid_dimensions);
        #endif
        src_color.a = 1.0;
        COLOR = src_color;
        //frag_color = src_color;
    }
}

// SPACE DENOMINATIONS:
//1. CANVAS NORMAL UV
//  - frag_coord ∈ [0…iResolution.xy] (pixels)
//  - uv = frag_coord / iResolution.y ∈ [0…(iResolution.x/iResolution.y)]×[0…1]

//2. LOCAL CELL NORMAL
//  - local_cell_progress_x/y ∈ [0…1]
//  - used by sample_uv_in_grid_space to pick which texel and apply hyperbolic/cubic warps.

//3. SUBPIXEL NORMAL
//  - store a normalized position ∈ [0…1) in current_*_subpixel_position
//  - then subtract LOCAL_CELL_SPACE_CENTER_OFFSET (0.5) → [-0.5…+0.5] so that

// IF ALL floating point numbers are integer_part + fraction_part: e.g. 1.5 = 1 + (1/2) = 1 + 0.5
// then to get the fraction part you just do:
//float fract(float x) {
    //int integer_part = floor(x)
    //float fraction_part = x - integer_part
    //return fraction_part;
//}


//https://en.wikipedia.org/wiki/Supersampling
// ^^ check out the techniques!!
