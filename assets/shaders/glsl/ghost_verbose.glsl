#version 100
precision mediump float;

varying vec2 fragTexCoord;
varying vec4 fragColor;

uniform float iTime;
uniform vec2  iResolution;

uniform sampler2D iChannel0;

#define HALF 0.5
#define GRID_SCALE 2.0
#define GRID_CELL_SIZE (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS vec2(0.5, 0.5)
#define GRID_ORIGIN_UV_OFFSET ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)
#define CELL_DRIFT_AMPLITUDE 0.2
#define LIGHT_WAVE_SPATIAL_FREQ_X 8.0
#define LIGHT_WAVE_SPATIAL_FREQ_Y 8.0
#define LIGHT_WAVE_TEMPORAL_FREQ_X 80.0
#define LIGHT_WAVE_TEMPORAL_FREQ_Y 2.3
#define LIGHT_WAVE_AMPLITUDE_X 0.0
#define LIGHT_WAVE_AMPLITUDE_Y 0.1
#define UMBRAL_MASK_OUTER_RADIUS 0.40 // bad name this is not for the umbral mask but rather the whole grid!
#define UMBRAL_MASK_INNER_RADIUS 0.08
#define UMBRAL_MASK_FADE_BAND 0.025
#define UMBRAL_MASK_CENTER vec2(HALF, HALF)
#define UMBRAL_MASK_OFFSET_X (-UMBRAL_MASK_OUTER_RADIUS / 1.0)
#define UMBRAL_MASK_OFFSET_Y (-UMBRAL_MASK_OUTER_RADIUS)
#define UMBRAL_MASK_PHASE_COEFFICIENT_X 0.6
#define UMBRAL_MASK_PHASE_COEFFICIENT_Y 0.2
#define UMBRAL_MASK_WAVE_AMPLITUDE_X 0.1
#define UMBRAL_MASK_WAVE_AMPLITUDE_Y 0.1

#define DITHER_TEXTURE_SCALE 8.0
#define DITHER_BLEND_FACTOR 0.75

const vec4 BLACK               = vec4(0.0, 0.0, 0.0, 1.0);
const vec4 LCARS_BLUEY         = vec4(0.53, 0.6, 1.0, 1.0);
const vec4 LCARS_GOLDEN_ORANGE = vec4(1.0, 0.6, 0.0, 1.0);

vec2 uv_to_grid_space(vec2 uv, float time) {
    uv               = uv - GRID_ORIGIN_UV_OFFSET;
    vec2 grid_coords = uv * GRID_SCALE;
    return grid_coords;
}

vec2 warp_and_drift_cell(vec2 grid_coords, float time) { return CELL_DRIFT_AMPLITUDE * sin(time + grid_coords.yx); }

vec2 spatial_phase(vec2 grid_coords) {
    return vec2(grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X, grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y);
}

vec2 temporal_phase(float time) { return vec2(time * LIGHT_WAVE_TEMPORAL_FREQ_X, time * LIGHT_WAVE_TEMPORAL_FREQ_Y); }

vec2 add_phase(vec2 phase) {
    float offset_x = LIGHT_WAVE_AMPLITUDE_X * cos(phase.x);
    float offset_y = LIGHT_WAVE_AMPLITUDE_Y * sin(phase.y);
    return vec2(offset_x, offset_y);
}

vec4 light_radial_fade(vec2 grid_coords, vec2 center, float radius, float feather) {
    float distance_from_center = length(grid_coords - center);
    float fade_start           = radius - feather;
    float alpha                = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    vec4  lightball            = vec4(clamp(alpha, 0.0, 1.0));
    return lightball;
}

vec2 add_umbral_mask_phase(float time) {
    vec2 phase = vec2(0.0);
    phase.x    = UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X;
    phase.y    = UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + time * LIGHT_WAVE_TEMPORAL_FREQ_Y;
    return phase;
}

vec2 umbral_mask_position(float x_phase_coefficient, float y_phase_coefficient, vec2 mask_phase) {
    float mask_pos_x  = x_phase_coefficient * cos(mask_phase.x);
    float mask_pos_y  = y_phase_coefficient * sin(mask_phase.y);
    vec2  offset_mask = vec2(mask_pos_x, mask_pos_y) + UMBRAL_MASK_CENTER;
    return offset_mask;
}

vec4 add_umbral_mask(vec4 src_color, vec2 grid_coords, vec2 mask_center) {
    vec2  mask_pos     = mask_center + vec2(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    float dist         = length(grid_coords - mask_pos);
    float half_dist    = dist * HALF;
    float mask         = smoothstep(UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OUTER_RADIUS, half_dist);
    vec4  applied_mask = src_color * mask;
    return applied_mask;
}

vec4 add_dither(vec4 src_color, vec2 frag_coord) {
    vec2  dither_uv      = frag_coord / DITHER_TEXTURE_SCALE;
    float dither_sample  = texture2D(iChannel0, dither_uv).r;
    vec4  dither_mask    = vec4(dither_sample);
    vec4  binary         = step(dither_mask, src_color);
    vec4  applied_dither = mix(src_color, binary, DITHER_BLEND_FACTOR);
    return applied_dither;
}

vec4 add_contour_overlay(vec4 src_color, vec2 grid_coords) {
    vec2 cell_idx = floor(grid_coords);
    if (any(notEqual(cell_idx, vec2(0.0, 0.0))))
        return src_color;
    vec2        p         = fract(grid_coords);
    float       d         = length(p - UMBRAL_MASK_CENTER) - UMBRAL_MASK_OUTER_RADIUS;
    const float spacing   = 0.08;
    const float thickness = 0.01;
    float       m         = mod(d + spacing * 0.5, spacing) - spacing * 0.5;
    float       mask      = smoothstep(thickness, 0.0, abs(m));
    float       inside    = step(length(p - UMBRAL_MASK_CENTER), UMBRAL_MASK_OUTER_RADIUS);
    float       alpha     = mask * inside * 0.5;
    return mix(src_color, LCARS_GOLDEN_ORANGE, alpha);
}

vec4 add_grid_overlay(vec4 src_color, vec2 grid_coords) {
    vec2        cell      = fract(grid_coords);
    float       edge      = min(min(cell.x, 1.0 - cell.x), min(cell.y, 1.0 - cell.y));
    const float lineWidth = 0.02;
    float       mask      = 1.0 - smoothstep(lineWidth * 0.5, lineWidth, edge);
    vec2        p         = fract(grid_coords);
    float       inside    = step(length(p - UMBRAL_MASK_CENTER), UMBRAL_MASK_OUTER_RADIUS);
    float       alpha     = mask * inside * 0.5;
    return mix(src_color, LCARS_BLUEY, alpha);
}
#define OUTLINE_THICKNESS 0.01
vec4 add_circle_outline(vec4 src_color, vec2 grid_coords) {
    vec2 cell_idx = floor(grid_coords);
    if (any(notEqual(cell_idx, vec2(0.0, 0.0))))
        return src_color;
    float dist  = length(fract(grid_coords) - UMBRAL_MASK_CENTER);
    float inner = UMBRAL_MASK_OUTER_RADIUS - OUTLINE_THICKNESS;
    float outer = UMBRAL_MASK_OUTER_RADIUS + OUTLINE_THICKNESS;
    float mask  = smoothstep(inner, UMBRAL_MASK_OUTER_RADIUS, dist) - smoothstep(UMBRAL_MASK_OUTER_RADIUS, outer, dist);
    return mix(src_color, LCARS_BLUEY, mask);
}

#define PI 3.14159265
#define INV_PI (1.0 / PI)
#define INV_TWO_PI (1.0 / (2.0 * PI))
#define WIRE_ROT_SPEED 0.5
#define WIRE_SEGMENT_COUNT 8.0
#define WIRE_SMOOTH_EDGE 0.05
#define USE_HORIZONTAL_ROTATE 0.0

vec4 add_wireframe_overlay(vec4 src_color, vec2 grid_coords) {
    vec2 cell_idx = floor(grid_coords);
    if (any(notEqual(cell_idx, vec2(0.0, 0.0))))
        return src_color;

    vec2  cell_uv   = fract(grid_coords);
    vec2  delta_xy  = cell_uv - UMBRAL_MASK_CENTER;
    float radius_sq = dot(delta_xy, delta_xy);
    float radius    = UMBRAL_MASK_OUTER_RADIUS;
    if (radius_sq > radius * radius) {
        return src_color;
    }
    float z_coord     = sqrt(radius * radius - radius_sq);
    vec3  normal_base = normalize(vec3(delta_xy, z_coord));
    float angle       = iTime * WIRE_ROT_SPEED;
    vec3  normal_h    = normalize(vec3(cos(angle) * normal_base.x + sin(angle) * normal_base.z, normal_base.y,
            -sin(angle) * normal_base.x + cos(angle) * normal_base.z));
    vec3  normal_v    = normalize(vec3(normal_base.x, cos(angle) * normal_base.y - sin(angle) * normal_base.z,
            sin(angle) * normal_base.y + cos(angle) * normal_base.z));
    vec3  normal_vec  = normalize(mix(normal_v, normal_h, USE_HORIZONTAL_ROTATE));
    float u_coord     = atan(normal_vec.z, normal_vec.x) * INV_TWO_PI + HALF;
    float v_coord     = acos(normal_vec.y) * INV_PI;
    float mu          = abs(mod(u_coord * WIRE_SEGMENT_COUNT, 1.0) - HALF);
    float mv          = abs(mod(v_coord * WIRE_SEGMENT_COUNT, 1.0) - HALF);
    float mask_u      = smoothstep(WIRE_SMOOTH_EDGE, 0.0, mu);
    float mask_v      = smoothstep(WIRE_SMOOTH_EDGE, 0.0, mv);
    float mask        = max(mask_u, mask_v);
    return mix(src_color, LCARS_GOLDEN_ORANGE, mask);
}
void main() {
    float time        = iTime;
    vec2  fragCoord   = fragTexCoord * iResolution;
    vec2  grid_coords = uv_to_grid_space(fragTexCoord, time);
    vec2  grid_phase  = vec2(0.0);
    grid_phase += spatial_phase(grid_coords);
    grid_phase += temporal_phase(time);
    grid_coords += add_phase(grid_phase);
    // grid_coords += warp_and_drift_cell(grid_coords, time);

    vec4 lightball
        = light_radial_fade(grid_coords, UMBRAL_MASK_CENTER, UMBRAL_MASK_OUTER_RADIUS, UMBRAL_MASK_FADE_BAND);
    vec4 src_color = lightball;

    // set up umbral mask to be drawn on top of transformed light ball
    vec2 umbral_mask_phase = vec2(0.0);
    umbral_mask_phase += add_umbral_mask_phase(time);
    vec2 umbral_mask_pos
        = umbral_mask_position(UMBRAL_MASK_PHASE_COEFFICIENT_X, UMBRAL_MASK_PHASE_COEFFICIENT_Y, umbral_mask_phase);

    //src_color = add_umbral_mask(src_color, grid_coords, umbral_mask_pos);
    //src_color = add_dither(src_color, fragCoord);
    // src_color    = add_contour_overlay(src_color, grid_coords);
    src_color    = add_wireframe_overlay(src_color, grid_coords);
    src_color    = add_circle_outline(src_color, grid_coords);
    src_color.a  = 1.0;
    gl_FragColor = src_color;
}
