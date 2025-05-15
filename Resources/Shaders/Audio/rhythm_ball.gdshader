shader_type canvas_item;
render_mode blend_disabled;

uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;
uniform sampler2D iChannel1: repeat_disable, filter_nearest;

uniform float bpm;
uniform vec2 time_signature;
uniform int   subdivisions_per_beat;        // e.g. 1,2,4,8…
uniform float seconds_per_beat;             // = 60.0 / bpm
uniform float seconds_per_subdivision;      // = seconds_per_beat / subdivisions_per_beat
uniform float seconds_per_bar;              // = seconds_per_beat * time_signature_numerator

#define HALF                        0.5
#define GRID_SCALE                  4.0
#define GRID_CELL_SIZE              (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX           vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS    vec2(5.66, 2.66)
#define GRID_ORIGIN_UV_OFFSET       ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)
#define CELL_DRIFT_AMPLITUDE        0.2
#define LIGHT_WAVE_SPATIAL_FREQ_X   8.0
#define LIGHT_WAVE_SPATIAL_FREQ_Y   8.0
#define LIGHT_WAVE_AMPLITUDE_X      0.03
#define LIGHT_WAVE_AMPLITUDE_Y      0.1
#define LIGHTBALL_OUTER_RADIUS      0.40
#define LIGHTBALL_FADE_BAND         0.025
#define LIGHTBALL_CENTER            vec2(HALF, HALF)
#define DITHER_TEXTURE_SCALE        8.0
#define DITHER_BLEND_FACTOR         0.75

#define total_fft_buffer_size_in_bins 512.0
#define WHITE                         vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK                         vec4(0.0, 0.0, 0.0, 1.0)
#define FFT_ROW                       0.0

// ——— Lightball/Grid Functions ———
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

vec2 add_phase(vec2 phase) {
    float offset_x = LIGHT_WAVE_AMPLITUDE_X * cos(phase.x);
    float offset_y = LIGHT_WAVE_AMPLITUDE_Y * sin(phase.y);
    return vec2(offset_x, offset_y);
}

vec4 light_radial_fade(vec2 grid_coords, vec2 center, float radius, float feather) {
    float distance_from_center = length(grid_coords - center);
    float fade_start = radius - feather;
    float alpha = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    vec4 lightball = vec4(clamp(alpha, 0.0, 1.0));
    return lightball;
}

vec4 add_dither(vec4 src_color, vec2 frag_coord) {
    vec2 dither_uv = frag_coord / DITHER_TEXTURE_SCALE;
    float dither_sample = texture(iChannel0, dither_uv).r;
    vec4 dither_mask = vec4(dither_sample);
    vec4 binary = step(dither_mask, src_color);
    vec4 applied_dither = mix(src_color, binary, DITHER_BLEND_FACTOR);
    return applied_dither;
}

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    // 1) FFT histogram across the full screen
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
            fft_color = WHITE;
        }
    }
    COLOR = fft_color;
    //frag_color = fft_color;

    // 2) Drifting lightball in the top-right grid cell
    float time = TIME;
    //float time = iTime;
    vec2 uv = frag_coord.xy / vec2(iResolution.y);

    vec2 grid_coords = uv_to_grid_space(uv, time);
    vec2 grid_phase  = spatial_phase(grid_coords);
    //grid_coords     += add_phase(grid_phase);
    //grid_coords     += warp_and_drift_cell(grid_coords, time);
    float phase = 0.0;
    //TODO: the below are all weird
    //phase = fract(time * bpm / 60.0);
    //phase = fract(time / seconds_per_beat);
    //phase = fract(time / seconds_per_subdivision);
    //phase = fract(time / seconds_per_bar);

    float beat_phase = fract(time / seconds_per_beat);
    float sub_beat_phase  = fract(time / seconds_per_subdivision);
    phase = mix(beat_phase, sub_beat_phase, HALF);
    
    float PULSE_MIN = 0.8;
    float PULSE_MAX = 1.2;
    float BPM_PULSE = mix(PULSE_MIN, PULSE_MAX, HALF + HALF * sin(phase * 6.2831853));
    vec4 lightball = light_radial_fade(
        grid_coords,
        LIGHTBALL_CENTER,
        LIGHTBALL_OUTER_RADIUS * BPM_PULSE,
        LIGHTBALL_FADE_BAND
    );
    
    //float pos_in_bar = mod(time, seconds_per_bar);
    //float f_beat = floor(pos_in_bar / seconds_per_beat);
    //int beatIndex = int(f_beat + 0.5);
    //float R_BIG   = 0.5;
    //float R_SMALL = 0.3;
    //float radius = R_SMALL;
    //if (beatIndex == 0) {
        //radius = R_BIG;
    //}
    //lightball = light_radial_fade(
        //grid_coords,
        //LIGHTBALL_CENTER,
        //radius,
        //LIGHTBALL_FADE_BAND
    //);
    vec4 src_color = add_dither(lightball, frag_coord);

    // overlay the lightball over the FFT histogram
    //frag_color = max(frag_color, src_color);
    //frag_color.a = 1.0;
    COLOR = max(COLOR, src_color);
    COLOR.a = 1.0;
}
