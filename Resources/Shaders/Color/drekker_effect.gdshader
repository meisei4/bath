shader_type canvas_item;
//this is a verbose study of cotterrzz max drecker reproduction https://www.shadertoy.com/view/33SSDm
uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_linear_mipmap, repeat_enable;

const int   SUPERSAMPLE_COUNT                 = 64;
const float SUPERSAMPLE_COUNT_F               = float(SUPERSAMPLE_COUNT);
const vec2  WEYL_SEQUENCE_STEP                = vec2(0.754877669, 0.569840296);
const float SUBPIXEL_CENTER_OFFSET            = 0.5;
const vec2  WEYL_SEQUENCE_JITTER_INITIAL_STATE = vec2(SUBPIXEL_CENTER_OFFSET);

const float TEXTURE_LOD_BASE_LEVEL        = 0.0;
const float DEFAULT_GRID_COLUMNS          = 11.0;
const float DEFAULT_GRID_ROWS             = 7.0;
const float GRID_CELLS_OUTER_PADDING        = 0.10;
const float GRID_CELLS_LATERAL_MARGIN_LEFT  = 0.10;
const float GRID_CELLS_LATERAL_MARGIN_RIGHT = 0.90;

const float STAGGER_FACTOR                = 0.10;
const float STROKE_CURVATURE              = 0.01;
const float THICKNESS_FALLOFF_SCALE       = 100.0;


#define CLEAR_CELL_BORDERS_ALPHA(color, x, y) {              \
            if ( abs(y) > GRID_CELLS_OUTER_PADDING           \
               || (x)   < GRID_CELLS_LATERAL_MARGIN_LEFT     \
               || (x)   > GRID_CELLS_LATERAL_MARGIN_RIGHT) { \
                (color).a = 0.0;                            \
            }                                               \
        }

vec2 compute_grid_dimensions() {
    return vec2(DEFAULT_GRID_COLUMNS, DEFAULT_GRID_ROWS);
}

float compute_discrete_column_index(float uv_x, float column_count) {
    float scaled_x       = uv_x * column_count;
    float scaled_x_floor = floor(scaled_x);
    return scaled_x_floor;
}

float compute_cell_progress_x(float uv_x, float column_count) {
    float scaled_x          = uv_x * column_count;
    float scaled_x_fraction = fract(scaled_x);
    return scaled_x_fraction;
}

float compute_vertical_curvature_hyperbolic(float cell_progress_x) {
    float left_curve_downwards    = (STROKE_CURVATURE / cell_progress_x) - STROKE_CURVATURE;
    float inverse_cell_progress_x = 1.0 - cell_progress_x;
    float right_curve_upwards     = (STROKE_CURVATURE / inverse_cell_progress_x) - STROKE_CURVATURE;
    float curvature               = left_curve_downwards - right_curve_upwards;
    return curvature;
}

float compute_discrete_row_index(float uv_y, float column_index, vec2 grid_dimensions, float curvature) {
    float staggered_y      = uv_y + column_index * STAGGER_FACTOR;
    float base_y           = staggered_y * grid_dimensions.y;
    float warped_y         = base_y + curvature;
    float floored_warped_y = floor(warped_y);

    return floored_warped_y;
}

float compute_cell_progress_y(float uv_y,float column_index, vec2 grid_dimensions,float curvature) {
    float staggered_y     = uv_y + column_index * STAGGER_FACTOR;
    float base_y          = staggered_y * grid_dimensions.y;
    float warped_y        = base_y + curvature;
    float warped_y_offset = fract(warped_y) - SUBPIXEL_CENTER_OFFSET;
    return warped_y_offset;
}

float compute_cubic_thickness_offset(float cell_progress_y) {
    return cell_progress_y * cell_progress_y * cell_progress_y;
}

vec4 sample_grid_at_fragment(vec2 uv, vec2 grid_dimensions) {
    float column_index           = compute_discrete_column_index(uv.x, grid_dimensions.x);
    float cell_progress_x        = compute_cell_progress_x(uv.x, grid_dimensions.x);
    float vertical_curvature     = compute_vertical_curvature_hyperbolic(cell_progress_x);
    float row_index              = compute_discrete_row_index(uv.y, column_index, grid_dimensions, vertical_curvature);
    float cell_progress_y        = compute_cell_progress_y(uv.y, column_index, grid_dimensions, vertical_curvature);
    float cubic_thickness_offset = compute_cubic_thickness_offset(cell_progress_y);

    float column_normal             = column_index / grid_dimensions.x;
    float row_normal                = row_index / grid_dimensions.y;
    float vertical_thickness        = cubic_thickness_offset / iResolution.y;
    float scaled_vertical_thickness = vertical_thickness * THICKNESS_FALLOFF_SCALE;
    float stagger_cells_vertically  = column_index * STAGGER_FACTOR;

    float u = column_normal;
    float v = row_normal + scaled_vertical_thickness - stagger_cells_vertically;

    vec4 color = textureLod(iChannel0, vec2(u,v), TEXTURE_LOD_BASE_LEVEL);

    CLEAR_CELL_BORDERS_ALPHA(color, cell_progress_x, cell_progress_y);

    return color;
}

vec2 advance_weyl_sequence(vec2 current_sequence_state) {
    vec2 next_sequence_state          = current_sequence_state + WEYL_SEQUENCE_STEP.yx;
    vec2 next_sequence_state_fraction = fract(next_sequence_state);
    return next_sequence_state_fraction;
}

vec4 super_sample_with_weyl_sequence_jitter(vec2 frag_coord, vec2 grid_dimensions) {
    vec4 weyl_sequence_jitter   = vec4(0.0);
    int  remaining_samples      = SUPERSAMPLE_COUNT;
    vec2 current_sequence_state = WEYL_SEQUENCE_JITTER_INITIAL_STATE;

    while (remaining_samples > 0) {
        current_sequence_state = advance_weyl_sequence(current_sequence_state);
        vec2 jitter_offset     = current_sequence_state - SUBPIXEL_CENTER_OFFSET;
        vec2 sample_coord      = frag_coord + jitter_offset;
        vec2 uv                = sample_coord / iResolution.y;
        weyl_sequence_jitter   = weyl_sequence_jitter + sample_grid_at_fragment(uv, grid_dimensions);
        remaining_samples      = remaining_samples - 1;
    }

    return weyl_sequence_jitter;
}


#define BORDER_CLEARING
void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv              = UV;
    vec2 frag_coord      = uv * iResolution;

    //vec2 uv            = frag_coord / iResolution.y;
    vec2 grid_dimensions = compute_grid_dimensions();
    vec4 color = sample_grid_at_fragment(uv, grid_dimensions);
    #ifdef BORDER_CLEARING
    if (color.a > 0.0) {
        COLOR = color;
        //frag_color = color;
    } else
    #endif //HACK...
    {
        //color = super_sample_with_weyl_sequence_jitter(FRAGCOORD.xy, grid_dimensions);
        color = super_sample_with_weyl_sequence_jitter(frag_coord, grid_dimensions);
        color = color / SUPERSAMPLE_COUNT_F;
        COLOR = color;
        //frag_color = color;
    }
}
