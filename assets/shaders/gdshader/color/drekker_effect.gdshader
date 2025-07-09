shader_type canvas_item;
#include "res://assets/shaders/gdshader/color/supersampling.gdshaderinc"

uniform vec2      iResolution;
uniform sampler2D iChannel0 : filter_nearest_mipmap, repeat_enable;

// TODO: this scalar is suppose to shift the cells but it has a phase/period where it just loops in on itself
//  Im am not sure the bounds to make it fully reprpoduce a purely unique shift cycle
// #define LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR    0.125
#define LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR 0.0125

// TODO: try different amplitudes and perhaps figure out different curvature functions to work with
#define HYPERBOLIC_AMPLITUDE 0.008
float hyper(float column_index) {
    float hyper = HYPERBOLIC_AMPLITUDE;
    return hyper;
}
#define CUBIC_SQUISH_SCALAR 90.0
#define NORMAL_OFFSET_MAIN 0.5

// #define DEBUG
#define DEBUG_LINE_THICKNESS_INNER 30.0
#define DEBUG_LINE_THICKNESS_OUTER 10.0
#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define CLEAR vec4(0.0, 0.0, 0.0, 0.0)
#define GREENISH vec4(0.5, 1.0, 0.5, 1.0)
#define VIOLET vec4(1.0, 0.0, 0.5, 1.0)

vec4 debug_cell_border(float local_cell_progress_x, float local_cell_progress_y) {
    float bw         = DEBUG_LINE_THICKNESS_OUTER / iResolution.y;
    bool  nearLeft   = local_cell_progress_x < bw;
    bool  nearRight  = local_cell_progress_x > 1.0 - bw;
    bool  nearBottom = local_cell_progress_y < bw;
    bool  nearTop    = local_cell_progress_y > 1.0 - bw;
    if (nearLeft || nearRight || nearBottom || nearTop) {
        return WHITE;
    }
    return CLEAR;
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
float compute_horizontal_hyperbolic_curvature(float hyperbolic_amplitude, float cell_progress_x) {
    float left_curve_downwards = hyperbolic_amplitude * ((1.0 / cell_progress_x) - 1.0);
    float right_curve_upwards  = hyperbolic_amplitude * ((1.0 / (1.0 - cell_progress_x)) - 1.0);
    float curvature            = left_curve_downwards - right_curve_upwards;
    return curvature;
}

vec4 debug_hyper_plot(float column_index, float local_x, float local_y) {
    float curv  = compute_horizontal_hyperbolic_curvature(hyper(column_index), local_x);
    float normC = clamp((curv / HYPERBOLIC_AMPLITUDE + 1.0) * 0.5, 0.0, 1.0);

    float inner = DEBUG_LINE_THICKNESS_INNER / iResolution.y;
    float outer = DEBUG_LINE_THICKNESS_OUTER / iResolution.y;
    float d     = abs(local_y - normC);

    float dashCount = 50.0;
    bool  dash      = mod(floor(local_x * dashCount), 2.0) < 1.0;

    if (!dash) {
        if (d < inner) {
            return GREENISH;
        } else if (d < outer) {
            return CLEAR;
        }
    }
    return CLEAR;
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
–0.5 ---------------+------------------ +0.5　<---from--the--right--- I think that you imagine this is the diretcion you
are looking at this curve in 3d somehow
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

float compute_vertical_cubic_color_squish(float local_cell_progress_y) {
    return local_cell_progress_y * local_cell_progress_y * local_cell_progress_y;
}

vec4 debug_cubic_plot(float local_x, float local_y) {
    float phaseY = local_y - NORMAL_OFFSET_MAIN;
    float squish = compute_vertical_cubic_color_squish(phaseY);
    float normT  = clamp(squish, 0.0, 1.0);

    float inner = DEBUG_LINE_THICKNESS_INNER / iResolution.y;
    float outer = DEBUG_LINE_THICKNESS_OUTER / iResolution.y;
    float d     = abs(local_y - normT);
    bool  dash  = mod(floor(local_x * 50.0), 2.0) < 1.0;

    if (d < inner && dash) {
        return VIOLET;
    } else if (d < outer) {
        return CLEAR;
    }
    return CLEAR;
}

vec4 sample_uv_in_grid_space(vec2 uv, vec2 grid_dimensions) {
    float column_index          = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float local_cell_progress_x = compute_local_cell_progress_x(uv.x, grid_dimensions.x);
    float horizontal_hyperbolic_curvature
        = compute_horizontal_hyperbolic_curvature(hyper(column_index), local_cell_progress_x);
    // float horizontal_hyperbolic_curvature = 0.0;

    float vertical_row_shift_per_column = LOCAL_CELL_SPACE_VERTICAL_SHIFT_SCALAR * column_index;
    float row_index                     = compute_discrete_row_index(
        uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);
    float local_cell_progress_y = compute_local_cell_progress_y(
        uv.y, vertical_row_shift_per_column, grid_dimensions.y, horizontal_hyperbolic_curvature);

    float phase_shifted_local_cell_progress_y = local_cell_progress_y - NORMAL_CENTER_OFFSET;
    float cubic_color_squish_cell_space       = compute_vertical_cubic_color_squish(local_cell_progress_y);
    // float cubic_color_squish_cell_space     = 0.0;
    float cubic_color_squish_normal = cubic_color_squish_cell_space / iResolution.y;
    float cubic_color_squish_scaled = CUBIC_SQUISH_SCALAR * cubic_color_squish_normal;

    float column_normal = column_index / grid_dimensions.x;
    float row_normal    = row_index / grid_dimensions.y;
    float u             = column_normal;
    float v             = row_normal + cubic_color_squish_scaled - vertical_row_shift_per_column;
    // vec4 texture_color = textureLod(iChannel0, vec2(u,v), TEXTURE_LOD_BASE_LEVEL);
    vec4 texture_color = texture(iChannel0, vec2(u, v));

    ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(texture_color, local_cell_progress_x, local_cell_progress_y);

    vec4 src_color = texture_color;

#ifdef DEBUG
    vec4 border_debug    = debug_cell_border(local_cell_progress_x, local_cell_progress_y);
    vec4 hyperplot_debug = debug_hyper_plot(column_index, local_cell_progress_x, local_cell_progress_y);
    vec4 cubicplot_debug = debug_cubic_plot(local_cell_progress_x, local_cell_progress_y);
    src_color.rgb        = mix(src_color.rgb, border_debug.rgb, border_debug.a);
    src_color.rgb        = mix(src_color.rgb, hyperplot_debug.rgb, hyperplot_debug.a);
    src_color.rgb        = mix(src_color.rgb, cubicplot_debug.rgb, cubicplot_debug.a);
    // NOTE on mix again..
    /*
    src_color.rgb = mix(src_color.rgb, hyperplot_debug.rgb,  hyperplot_debug.a);

    ^this is the same as the following:

    float a = hyperplot_debug.a;
    src_color.r = src_color.r * (1.0 - a) + hyperplot_debug.r * a;
    src_color.g = src_color.g * (1.0 - a) + hyperplot_debug.g * a;
    src_color.b = src_color.b * (1.0 - a) + hyperplot_debug.b * a;
    */
#endif
    ALPHA_CLEAR_TEXELS_OUTSIDE_CELL_BOUNDARIES(src_color, local_cell_progress_x, local_cell_progress_y);
    return src_color;
}

vec4 uniform_supersample(vec2 frag_coord, vec2 grid_dimensions) {
    vec4 uniform_distribution               = vec4(0.0);
    int  remaining_subpixels_in_supersample = UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL;
    vec2 current_uniform_subpixel_position  = INITIAL_UNIFORM_STEP;
    while (remaining_subpixels_in_supersample > 0) {
        current_uniform_subpixel_position  = UNIFORM_STEP(current_uniform_subpixel_position);
        vec2 sample_coord                  = frag_coord + current_uniform_subpixel_position;
        vec2 uv                            = sample_coord / iResolution.y;
        uniform_distribution               = uniform_distribution + sample_uv_in_grid_space(uv, grid_dimensions);
        remaining_subpixels_in_supersample = remaining_subpixels_in_supersample - 1;
    }
    return uniform_distribution / UNIFORM_SUPERSAMPLE_RESOLUTION_SUBPIXEL_SAMPLES_PER_PIXEL_F;
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
// #define SUPERSAMPLE(frag_coord, grid_dimensions) uniform_supersample(frag_coord, grid_dimensions)

void fragment() {
    vec2 frag_coord      = vec2(UV.x * iResolution.x, UV.y * iResolution.y);
    vec2 uv              = frag_coord / iResolution.y;
    vec2 grid_dimensions = compute_grid_dimensions();
    vec4 src_color       = sample_uv_in_grid_space(uv, grid_dimensions);
    if (src_color.a > 0.0) {
        // src_color.a = 1.0; // HARDCODE STRAIGHT ALPHA BLEND TO NOT OCCUR (i.e. destination.rgb * 0.0)
        src_color.a = 1.0;
        COLOR       = src_color;
    } else {
        src_color   = SUPERSAMPLE(frag_coord, grid_dimensions);
        src_color.a = 1.0;
        COLOR       = src_color;
    }
}

// SPACE DENOMINATIONS:
// 1. CANVAS NORMAL UV
//  - frag_coord ∈ [0…iResolution.xy] (pixels)
//  - uv = frag_coord / iResolution.y ∈ [0…(iResolution.x/iResolution.y)]×[0…1]

// 2. LOCAL CELL NORMAL
//   - local_cell_progress_x/y ∈ [0…1]
//   - used by sample_uv_in_grid_space to pick which texel and apply hyperbolic/cubic warps.

// 3. SUBPIXEL NORMAL
//   - store a normalized position ∈ [0…1) in current_*_subpixel_position
//   - then subtract LOCAL_CELL_SPACE_CENTER_OFFSET (0.5) → [-0.5…+0.5] so that

// IF ALL floating point numbers are integer_part + fraction_part: e.g. 1.5 = 1 + (1/2) = 1 + 0.5
// then to get the fraction part you just do:
// float fract(float x) {
// int integer_part = floor(x)
// float fraction_part = x - integer_part
// return fraction_part;
//}

// https://en.wikipedia.org/wiki/Supersampling
//  ^^ check out the techniques!!
