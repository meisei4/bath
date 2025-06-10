shader_type canvas_item;
render_mode blend_disabled;

uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;
uniform sampler2D iChannel1: repeat_disable, filter_nearest;
//uniform sampler2D iChannel2: repeat_disable, filter_nearest;
const int MAX_CUSTOM_ONSETS = 128;

uniform int f_onset_count;
uniform int j_onset_count;
uniform vec2 f_onsets[MAX_CUSTOM_ONSETS];
uniform vec2 j_onsets[MAX_CUSTOM_ONSETS];

uniform float bpm;
uniform float song_time;

uniform vec3 hsv_buffer[6];

#define TAU 6.283185307179586
#define HALF 0.5
#define GRID_SCALE               4.0
#define GRID_CELL_SIZE           (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX        vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS vec2(5.66, 2.33)
#define GRID_ORIGIN_UV_OFFSET    ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)
#define CELL_DRIFT_AMPLITUDE    0.2
#define LIGHT_WAVE_SPATIAL_FREQ_X     8.0
#define LIGHT_WAVE_SPATIAL_FREQ_Y     8.0
#define LIGHT_WAVE_TEMPORAL_FREQ_X    80.0
#define LIGHT_WAVE_TEMPORAL_FREQ_Y    2.3
#define LIGHT_WAVE_AMPLITUDE_X        0.03
#define LIGHT_WAVE_AMPLITUDE_Y        0.1
#define LIGHTBALL_OUTER_RADIUS        0.40
#define LIGHTBALL_CENTER              vec2(HALF, HALF)
#define LIGHTBALL_FADE_BAND           0.025
#define UMBRAL_MASK_INNER_RADIUS      0.08
#define UMBRAL_MASK_OFFSET_X         (-LIGHTBALL_OUTER_RADIUS /1.0)
#define UMBRAL_MASK_OFFSET_Y         (-LIGHTBALL_OUTER_RADIUS)
#define UMBRAL_MASK_PHASE_COEFFICIENT_X  0.6
#define UMBRAL_MASK_PHASE_COEFFICIENT_Y  0.2
#define UMBRAL_MASK_WAVE_AMPLITUDE_X     0.1
#define UMBRAL_MASK_WAVE_AMPLITUDE_Y     0.1
#define DITHER_TEXTURE_SCALE    8.0
#define DITHER_BLEND_FACTOR     0.75

#define JERK_WINDOW  0.05
#define JERK_UP_AMP  0.25
#define JERK_DN_AMP  0.4

#define total_fft_buffer_size_in_bins 512.0
#define WHITE                         vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK                         vec4(0.0, 0.0, 0.0, 1.0)
#define FFT_ROW                       0.0

vec2 uv_to_grid_space(vec2 uv, float time) {
    uv = uv - GRID_ORIGIN_UV_OFFSET;
    vec2 grid_coords = uv * GRID_SCALE;
    return grid_coords;
}

vec2 warp_and_drift_cell(vec2 grid_coords, float time) {
    return CELL_DRIFT_AMPLITUDE * sin(time + grid_coords.yx);
}

vec2 spatial_phase(vec2 grid_coords) {
    return vec2(
        grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X,
        grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y
    );
}

vec2 temporal_phase(float time) {
    return vec2(
        time * LIGHT_WAVE_TEMPORAL_FREQ_X,
        time * LIGHT_WAVE_TEMPORAL_FREQ_Y
    );
}

vec2 add_phase(vec2 phase) {
    float offset_x = LIGHT_WAVE_AMPLITUDE_X * cos(phase.x);
    float offset_y = LIGHT_WAVE_AMPLITUDE_Y * sin(phase.y);
    return vec2(offset_x, offset_y);
}

vec3 hsv_to_rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec4 light_radial_fade_hsv(vec2 grid_coords, vec2 center, float radius, float feather) {
    float distance_from_center = length(grid_coords - center);
    float fade_start = radius - feather;
    float alpha = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    vec3 hsv = hsv_buffer[0];
    hsv.z *= 5.0; // Boost brightness
    vec3 rgb = hsv_to_rgb(hsv);
    return vec4(rgb * clamp(alpha, 0.0, 1.0), 1.0);
}

vec4 light_radial_fade(vec2 grid_coords, vec2 center, float radius, float feather) {
    float distance_from_center = length(grid_coords - center);
    float fade_start = radius - feather;
    float alpha = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    vec4 lightball = vec4(clamp(alpha, 0.0, 1.0));
    return lightball;
}

vec2 add_umbral_mask_phase(float time){
    vec2 phase = vec2(0.0);
    phase.x = UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X;
    phase.y = UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + time * LIGHT_WAVE_TEMPORAL_FREQ_Y;
    return phase;
}

vec2 umbral_mask_position(float x_phase_coefficient, float y_phase_coefficient, vec2 mask_phase) {
    float mask_pos_x = x_phase_coefficient * cos(mask_phase.x);
    float mask_pos_y = y_phase_coefficient * sin(mask_phase.y);
    vec2 offset_mask = vec2(mask_pos_x, mask_pos_y) + LIGHTBALL_CENTER;
    return offset_mask;
}

vec4 add_umbral_mask(vec4 src_color, vec2 grid_coords, vec2 mask_center) {
    vec2  mask_pos  = mask_center + vec2(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    float dist        = length(grid_coords - mask_pos);
    float half_dist   = dist * HALF;
    float mask        = smoothstep(UMBRAL_MASK_INNER_RADIUS, LIGHTBALL_OUTER_RADIUS, half_dist);
    vec4  applied_mask = src_color * mask;
    return applied_mask;
}

vec4 add_dither(vec4 src_color, vec2 frag_coord) {
    vec2 dither_uv = frag_coord / DITHER_TEXTURE_SCALE;
    float dither_sample = texture(iChannel0, dither_uv).r;
    vec4 dither_mask = vec4(dither_sample);
    vec4 binary = step(dither_mask, src_color);
    vec4 applied_dither = mix(src_color, binary, DITHER_BLEND_FACTOR);
    return applied_dither;
}

float compute_radial_phase(float time) {
    // If bpm == 0.0, then 60.0 / 0.0 → +∞ (in IEEE‐754 floating point).
    float seconds_per_beat = 60.0 / bpm;
    // time / seconds_per_beat → finite_time / +∞ → 0.0
    // fract(0.0) → 0.0
    return fract(time / seconds_per_beat);
}

float pulse_radius(float time) {
    // compute_radial_phase(time) will be 0.0 whenever bpm == 0.0
    float phase = compute_radial_phase(time);
    const float PULSE_MIN = 0.8;
    const float PULSE_MAX = 1.2;
    // phase * 2π = 0.0 * 6.2831853 = 0.0
    // sin(0.0) = 0.0
    // HALF + HALF * 0.0 = 0.5
    float blend_factor = HALF + HALF * sin(phase * TAU);
    // mix(a, b, 0.5) always returns (a + b) / 2
    // mix(0.8, 1.2, 0.5) = (0.8 + 1.2) / 2 = 1.0
    float mul = mix(PULSE_MIN, PULSE_MAX, blend_factor);
    // LIGHTBALL_OUTER_RADIUS * 1.0 = LIGHTBALL_OUTER_RADIUS
    return LIGHTBALL_OUTER_RADIUS * mul;
}

vec2 jerk_uki_shizumi(vec2 grid_coords, float time) {
    float jerk = 0.0;
    for (int i = 0; i < MAX_CUSTOM_ONSETS; i++) {
        if (i >= f_onset_count) break;
        float f_press   = f_onsets[i].x;
        float f_release = f_onsets[i].y;
        float du = time - f_press;
        if (du >= 0.0 && du <= JERK_WINDOW) {
            float t = du / JERK_WINDOW;
            jerk += sin(t * 3.141592) * JERK_UP_AMP;
        }
        float dr = time - f_release;
        if (dr >= 0.0 && dr <= JERK_WINDOW) {
            float t2 = dr / JERK_WINDOW;
            jerk -= sin(t2 * 3.141592) * (JERK_UP_AMP * 0.5);
        }
    }
    for (int i = 0; i < MAX_CUSTOM_ONSETS; i++) {
        if (i >= j_onset_count) break;
        float j_press   = j_onsets[i].x;
        float j_release = j_onsets[i].y;
        float du2 = time - j_press;
        if (du2 >= 0.0 && du2 <= JERK_WINDOW) {
            float t3 = du2 / JERK_WINDOW;
            jerk -= sin(t3 * 3.141592) * JERK_DN_AMP;
        }
        float dr2 = time - j_release;
        if (dr2 >= 0.0 && dr2 <= JERK_WINDOW) {
            float t4 = dr2 / JERK_WINDOW;
            jerk += sin(t4 * 3.141592) * (JERK_DN_AMP * 0.5);
        }
    }
    grid_coords.y += jerk;
    return grid_coords;
}


vec4 fft_spectrum(vec2 frag_coord){
    vec2 uv_full = frag_coord.xy / iResolution.xy;
    float cell_width = iResolution.x / total_fft_buffer_size_in_bins;
    float bin_index  = floor(frag_coord.x / cell_width);
    float local_x    = mod(frag_coord.x, cell_width);
    float bar_width  = cell_width - 1.0;
    vec4 fft_color = BLACK;
    if (local_x <= bar_width) {
        float sample_x = (bin_index + 0.5) / total_fft_buffer_size_in_bins;
        float amplitude = texture(iChannel1, vec2(sample_x, FFT_ROW)).r;
        if (uv_full.y < amplitude) {
            vec3 hsv_fft = hsv_to_rgb(hsv_buffer[1]);
            hsv_fft.z *= 3.0;
            fft_color = vec4(hsv_fft, 1.0);
        }
    }
    return fft_color;
}

vec4 ghost(vec2 frag_coord, float time) {
    vec2 uv = frag_coord.xy / vec2(iResolution.y);
    vec2 grid_coords = uv_to_grid_space(uv, time);
    grid_coords      = jerk_uki_shizumi(grid_coords, time);
    vec2 grid_phase   = vec2(0.0);
    grid_phase       += spatial_phase(grid_coords);
    grid_phase       += temporal_phase(time);
    grid_coords      += add_phase(grid_phase);
    grid_coords      += warp_and_drift_cell(grid_coords, time);
    float radius = pulse_radius(time);
    vec4 lightball = light_radial_fade(
    //vec4 lightball = light_radial_fade_hsv(
            grid_coords,
            LIGHTBALL_CENTER,
            radius,
            LIGHTBALL_FADE_BAND
    );

    vec4 src_color    = lightball;
    vec2 umbral_mask_phase = vec2(0.0);
    umbral_mask_phase     += add_umbral_mask_phase(time);
    vec2 umbral_mask_pos   = umbral_mask_position(UMBRAL_MASK_PHASE_COEFFICIENT_X, UMBRAL_MASK_PHASE_COEFFICIENT_Y, umbral_mask_phase);
    src_color              = add_umbral_mask(src_color, grid_coords, umbral_mask_pos);
    src_color              = add_dither(src_color, frag_coord);
    return src_color;
}

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    //vec3 hsv_bg = hsv_to_rgb(hsv_buffer[1]);
    //COLOR = vec4(hsv_bg, 1.0);
    COLOR = BLACK;
    COLOR = fft_spectrum(frag_coord);
    vec4 src_color = ghost(frag_coord, song_time);
    COLOR = max(COLOR, src_color);
    COLOR.a = 1.0;
}
