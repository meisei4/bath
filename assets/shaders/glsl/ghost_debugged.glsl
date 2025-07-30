#version 100
precision mediump float;

varying vec2 fragTexCoord;
varying vec4 fragColor;

uniform float iTime;
uniform vec2  iResolution;

uniform sampler2D iChannel0;

#define TAU 6.283185307179586
#define HALF 0.5

#define GRID_SCALE 4.0
#define GRID_CELL_SIZE (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS vec2(2., 2.)
#define GRID_ORIGIN_UV_OFFSET ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)

#define CELL_DRIFT_AMPLITUDE 0.2
#define LIGHT_WAVE_SPATIAL_FREQ_X 8.0
#define LIGHT_WAVE_SPATIAL_FREQ_Y 8.0
#define LIGHT_WAVE_TEMPORAL_FREQ_X 80.0
#define LIGHT_WAVE_TEMPORAL_FREQ_Y 2.3
#define LIGHT_WAVE_AMPLITUDE_X 0.03
#define LIGHT_WAVE_AMPLITUDE_Y 0.10

#define LIGHTBALL_OUTER_RADIUS 0.40
#define LIGHTBALL_CENTER vec2(HALF, HALF)
#define LIGHTBALL_FADE_BAND 0.025

#define UMBRAL_MASK_INNER_RADIUS 0.08
#define UMBRAL_MASK_OFFSET_X (-LIGHTBALL_OUTER_RADIUS / 1.0)
#define UMBRAL_MASK_OFFSET_Y (-LIGHTBALL_OUTER_RADIUS)
#define UMBRAL_MASK_PHASE_COEFFICIENT_X 0.6
#define UMBRAL_MASK_PHASE_COEFFICIENT_Y 0.2
#define UMBRAL_MASK_WAVE_AMPLITUDE_X 0.1
#define UMBRAL_MASK_WAVE_AMPLITUDE_Y 0.1

#define DITHER_TEXTURE_SCALE 8.0
#define DITHER_BLEND_FACTOR 0.75

const vec4 BLACK = vec4(0.0, 0.0, 0.0, 1.0);

vec2 uv_to_grid_space(vec2 uv) { return (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE; }

vec2 warp_and_drift_cell(vec2 grid_coords) { return CELL_DRIFT_AMPLITUDE * sin(iTime + grid_coords.yx); }

vec2 spatial_phase(vec2 grid_coords) {
    return vec2(grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X, grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y);
}

vec2 temporal_phase() { return vec2(iTime * LIGHT_WAVE_TEMPORAL_FREQ_X, iTime * LIGHT_WAVE_TEMPORAL_FREQ_Y); }

vec2 add_phase(vec2 phase) {
    return vec2(LIGHT_WAVE_AMPLITUDE_X * cos(phase.x), LIGHT_WAVE_AMPLITUDE_Y * sin(phase.y));
}

vec4 light_radial_fade(vec2 grid_coords, vec2 center, float radius, float feather) {
    float dist  = length(grid_coords - center);
    float alpha = 1.0 - smoothstep(radius - feather, radius, dist);
    return vec4(vec3(clamp(alpha, 0.0, 1.0)), 1.0);
}

vec2 add_umbral_mask_phase() {
    return vec2(UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X,
        UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + iTime * LIGHT_WAVE_TEMPORAL_FREQ_Y);
}

vec2 umbral_mask_position(float x_coeff, float y_coeff, vec2 phase) {
    return LIGHTBALL_CENTER + vec2(x_coeff * cos(phase.x), y_coeff * sin(phase.y));
}

vec4 add_umbral_mask(vec4 src_color, vec2 grid_coords, vec2 mask_center) {
    vec2  mask_pos  = mask_center + vec2(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    float half_dist = length(grid_coords - mask_pos) * HALF;
    float m         = smoothstep(UMBRAL_MASK_INNER_RADIUS, LIGHTBALL_OUTER_RADIUS, half_dist);
    return src_color * m;
}

vec4 add_dither(vec4 src) {
    vec2  pixel = fragTexCoord * iResolution;
    vec2  dUV   = fract(pixel / DITHER_TEXTURE_SCALE);
    float th    = texture2D(iChannel0, dUV).r;
    float bit   = step(th, src.r);
    return mix(src, vec4(vec3(bit), 1.0), DITHER_BLEND_FACTOR);
}

void main() {
    vec2 grid_coords = uv_to_grid_space(fragTexCoord);
    vec2 grid_phase  = spatial_phase(grid_coords) + temporal_phase();
    grid_coords += add_phase(grid_phase) + warp_and_drift_cell(grid_coords);
    vec4 src_color  = light_radial_fade(grid_coords, LIGHTBALL_CENTER, LIGHTBALL_OUTER_RADIUS, LIGHTBALL_FADE_BAND);
    vec2 mask_phase = add_umbral_mask_phase();
    vec2 mask_pos = umbral_mask_position(UMBRAL_MASK_PHASE_COEFFICIENT_X, UMBRAL_MASK_PHASE_COEFFICIENT_Y, mask_phase);
    src_color     = add_umbral_mask(src_color, grid_coords, mask_pos);
    vec4 finalColor = add_dither(src_color);
    gl_FragColor    = finalColor;
}
