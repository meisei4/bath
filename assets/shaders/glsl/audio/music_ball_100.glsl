// #version 330
#version 100
precision mediump float;

// in vec2 fragTexCoord;
// in vec4 fragColor;
varying vec2 fragTexCoord;
varying vec4 fragColor;

// out vec4 finalColor;

uniform float iTime;
uniform vec2  iResolution;

uniform sampler2D iChannel0;
uniform sampler2D iChannel1;

const int MAX_CUSTOM_ONSETS = 16;

uniform vec2 f_onsets[MAX_CUSTOM_ONSETS];
uniform vec2 j_onsets[MAX_CUSTOM_ONSETS];

uniform float bpm;
uniform vec3  hsv_buffer[6];

#define TAU 6.283185307179586
#define PI 3.141592
#define HALF 0.5
#define GRID_SCALE 4.0
#define GRID_CELL_SIZE (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS vec2(5.66, 2.33)
#define GRID_ORIGIN_UV_OFFSET ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)
#define CELL_DRIFT_AMPLITUDE 0.2
#define LIGHT_WAVE_SPATIAL_FREQ_X 8.0
#define LIGHT_WAVE_SPATIAL_FREQ_Y 8.0
#define LIGHT_WAVE_TEMPORAL_FREQ_X 80.0
#define LIGHT_WAVE_TEMPORAL_FREQ_Y 2.3
#define LIGHT_WAVE_AMPLITUDE_X 0.03
#define LIGHT_WAVE_AMPLITUDE_Y 0.1
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

#define JERK_WINDOW 0.05
#define JERK_UP_AMP 0.25
#define JERK_DN_AMP 0.4

#define NUM_OF_BINS 512.0
#define FFT_ROW 0.0

const vec4 BLACK = vec4(0.0, 0.0, 0.0, 1.0);
const vec4 WHITE = vec4(1.0, 1.0, 1.0, 1.0);

vec2 uv_to_grid_space(vec2 uv, float iTime) {
    uv               = uv - GRID_ORIGIN_UV_OFFSET;
    vec2 grid_coords = uv * GRID_SCALE;
    return grid_coords;
}

vec2 warp_and_drift_cell(vec2 grid_coords, float iTime) { return CELL_DRIFT_AMPLITUDE * sin(iTime + grid_coords.yx); }

vec2 spatial_phase(vec2 grid_coords) {
    return vec2(grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X, grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y);
}

vec2 temporal_phase(float iTime) {
    return vec2(iTime * LIGHT_WAVE_TEMPORAL_FREQ_X, iTime * LIGHT_WAVE_TEMPORAL_FREQ_Y);
}

vec2 add_phase(vec2 phase) {
    float offset_x = LIGHT_WAVE_AMPLITUDE_X * cos(phase.x);
    float offset_y = LIGHT_WAVE_AMPLITUDE_Y * sin(phase.y);
    return vec2(offset_x, offset_y);
}

vec3 hsv_to_rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec4 light_radial_fade_hsv(vec2 grid_coords, vec2 center, float radius, float feather) {
    float distance_from_center = length(grid_coords - center);
    float fade_start           = radius - feather;
    float alpha                = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    vec3  hsv                  = hsv_buffer[0];
    hsv.z *= 5.0; // Boost brightness
    vec3 rgb = hsv_to_rgb(hsv);
    return vec4(rgb * clamp(alpha, 0.0, 1.0), 1.0);
}

vec4 light_radial_fade(vec2 grid_coords, vec2 center, float radius, float feather) {
    float distance_from_center = length(grid_coords - center);
    float fade_start           = radius - feather;
    float alpha                = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    vec4  lightball            = vec4(clamp(alpha, 0.0, 1.0));
    return lightball;
}

vec2 add_umbral_mask_phase(float iTime) {
    vec2 phase = vec2(0.0);
    phase.x    = UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X;
    phase.y    = UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + iTime * LIGHT_WAVE_TEMPORAL_FREQ_Y;
    return phase;
}

vec2 umbral_mask_position(float x_phase_coefficient, float y_phase_coefficient, vec2 mask_phase) {
    float mask_pos_x  = x_phase_coefficient * cos(mask_phase.x);
    float mask_pos_y  = y_phase_coefficient * sin(mask_phase.y);
    vec2  offset_mask = vec2(mask_pos_x, mask_pos_y) + LIGHTBALL_CENTER;
    return offset_mask;
}

vec4 add_umbral_mask(vec4 src_color, vec2 grid_coords, vec2 mask_center) {
    vec2  mask_pos     = mask_center + vec2(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    float dist         = length(grid_coords - mask_pos);
    float half_dist    = dist * HALF;
    float mask         = smoothstep(UMBRAL_MASK_INNER_RADIUS, LIGHTBALL_OUTER_RADIUS, half_dist);
    vec4  applied_mask = src_color * mask;
    return applied_mask;
}

vec4 add_dither(vec4 src_color, vec2 fragCoord) {
    vec2 dither_uv = fragCoord / DITHER_TEXTURE_SCALE;
    // float dither_sample  = texture(iChannel0, dither_uv).r;
    float dither_sample  = texture2D(iChannel0, dither_uv).r;
    vec4  dither_mask    = vec4(dither_sample);
    vec4  binary         = step(dither_mask, src_color);
    vec4  applied_dither = mix(src_color, binary, DITHER_BLEND_FACTOR);
    return applied_dither;
}

float compute_radial_phase(float iTime) {
    float seconds_per_beat = 60.0 / bpm;
    return fract(iTime / seconds_per_beat);
}

float pulse_radius(float iTime) {
    float       phase        = compute_radial_phase(iTime);
    const float PULSE_MIN    = 0.8;
    const float PULSE_MAX    = 1.2;
    float       blend_factor = HALF + HALF * sin(phase * TAU);
    float       mul          = mix(PULSE_MIN, PULSE_MAX, blend_factor);
    return LIGHTBALL_OUTER_RADIUS * mul;
}

vec2 jerk_uki_shizumi(vec2 grid_coords, float iTime) {
    // TODO: SIMD fixed uniform loop bounds, no breaks, no branching
    float     total_jerk_offset = 0.0;
    const int LOOPS             = MAX_CUSTOM_ONSETS * 2;
    for (int i = 0; i < LOOPS; ++i) {
        int onset_index = int(mod(float(i), float(MAX_CUSTOM_ONSETS)));
        // int   onset_index  = i % MAX_CUSTOM_ONSETS;
        float use_f_onsets = step(float(i), float(MAX_CUSTOM_ONSETS));

        vec2 f_pair     = f_onsets[onset_index];
        vec2 j_pair     = j_onsets[onset_index];
        vec2 onset_pair = mix(j_pair, f_pair, use_f_onsets);

        float up_amp = mix(-JERK_DN_AMP, JERK_UP_AMP, use_f_onsets);
        float dn_amp = mix(JERK_DN_AMP * 0.5, -JERK_UP_AMP * 0.5, use_f_onsets);

        float time_since_press   = iTime - onset_pair.x;
        float time_since_release = iTime - onset_pair.y;

        float press_in_window = step(0.0, time_since_press) * step(time_since_press, JERK_WINDOW);
        float press_t         = clamp(time_since_press / JERK_WINDOW, 0.0, 1.0);
        total_jerk_offset += press_in_window * sin(press_t * PI) * up_amp;

        float release_in_window = step(0.0, time_since_release) * step(time_since_release, JERK_WINDOW);
        float release_t         = clamp(time_since_release / JERK_WINDOW, 0.0, 1.0);
        total_jerk_offset += release_in_window * sin(release_t * PI) * dn_amp;
    }
    grid_coords.y += total_jerk_offset;
    return grid_coords;
}

vec4 fft_spectrum_branchless(vec2 fragCoord) {
    float cell_width = iResolution.x / NUM_OF_BINS;
    float bin_index  = floor(fragCoord.x / cell_width);
    float local_x    = mod(fragCoord.x, cell_width);
    float bar_width  = cell_width - 1.0;
    float sample_x   = (bin_index + 0.5) / NUM_OF_BINS;
    // float amplitude  = texture(iChannel1, vec2(sample_x, FFT_ROW)).r;
    float amplitude = texture2D(iChannel1, vec2(sample_x, FFT_ROW)).r;
    float bar_mask  = step(local_x, bar_width);
    float amp_mask  = step(fragTexCoord.y, amplitude);
    float in_bar    = bar_mask * amp_mask;
    vec3  hsv_fft   = hsv_to_rgb(hsv_buffer[0]) * 3.0;
    vec4  fft_color = vec4(hsv_fft * in_bar, 1.0);
    return fft_color;
}

vec4 ghost(vec2 fragCoord) {
    vec2 uv_y_aspect = fragCoord / vec2(iResolution.y);
    vec2 grid_coords = uv_to_grid_space(uv_y_aspect, iTime);
    grid_coords      = jerk_uki_shizumi(grid_coords, iTime);
    vec2 grid_phase  = vec2(0.0);
    grid_phase += spatial_phase(grid_coords);
    grid_phase += temporal_phase(iTime);
    grid_coords += add_phase(grid_phase);
    grid_coords += warp_and_drift_cell(grid_coords, iTime);
    float radius            = pulse_radius(iTime);
    vec4  lightball         = light_radial_fade(grid_coords, LIGHTBALL_CENTER, radius, LIGHTBALL_FADE_BAND);
    vec4  src_color         = lightball;
    vec2  umbral_mask_phase = vec2(0.0);
    umbral_mask_phase += add_umbral_mask_phase(iTime);
    vec2 umbral_mask_pos
        = umbral_mask_position(UMBRAL_MASK_PHASE_COEFFICIENT_X, UMBRAL_MASK_PHASE_COEFFICIENT_Y, umbral_mask_phase);
    src_color = add_umbral_mask(src_color, grid_coords, umbral_mask_pos);
    src_color = add_dither(src_color, fragCoord);
    return src_color;
}

void main() {
    vec2 fragCoord  = fragTexCoord * iResolution;
    vec4 finalColor = BLACK;
    finalColor      = fft_spectrum_branchless(fragCoord);
    vec4 src_color  = ghost(fragCoord);
    finalColor      = max(finalColor, src_color);
    finalColor.a    = 1.0;
    gl_FragColor    = finalColor;
}
