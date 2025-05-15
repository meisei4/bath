shader_type canvas_item;
render_mode blend_disabled;

uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;
uniform sampler2D iChannel1: repeat_disable, filter_nearest;
uniform sampler2D iChannel2: repeat_disable, filter_nearest;


uniform float onsets_per_minute;
//uniform vec2 time_signature;
//uniform int   subdivisions_per_beat;        // e.g. 1,2,4,8…
//uniform float seconds_per_beat;             // = 60.0 / bpm
//uniform float seconds_per_subdivision;      // = seconds_per_beat / subdivisions_per_beat
//uniform float seconds_per_bar;              // = seconds_per_beat * time_signature_numerator

#define HALF                        0.5
#define GRID_SCALE                  4.0
#define GRID_CELL_SIZE              (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX           vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS    vec2(5.66, 2.66)
#define GRID_ORIGIN_UV_OFFSET       ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)
#define LIGHTBALL_OUTER_RADIUS      0.40
#define LIGHTBALL_FADE_BAND         0.025
#define LIGHTBALL_CENTER            vec2(HALF, HALF)
#define DITHER_TEXTURE_SCALE        8.0
#define DITHER_BLEND_FACTOR         0.75

#define total_fft_buffer_size_in_bins 512.0
#define WHITE                         vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK                         vec4(0.0, 0.0, 0.0, 1.0)
#define FFT_ROW                       0.0

vec2 uv_to_grid_space(vec2 uv, float time) {
    uv = uv - GRID_ORIGIN_UV_OFFSET;
    vec2 grid_coords = uv * GRID_SCALE;
    return grid_coords;
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

float compute_radial_phase(float time) {
    //return fract(t * bpm / 60.0);
    //return fract(t / seconds_per_beat);
    //return fract(t / seconds_per_subdivision);
    //return fract(t / seconds_per_bar);
    //float beat_phase     = fract(time / seconds_per_beat);
    //float subdiv_phase   = fract(time / seconds_per_subdivision);
    //return mix(beat_phase, subdiv_phase, HALF);
    return 0.0;
}

float pulse_radius_experimental(float time) {
    float phase = compute_radial_phase(time);
    const float PULSE_MIN = 0.8;
    const float PULSE_MAX = 1.2;
    float mul = mix(PULSE_MIN,
                    PULSE_MAX,
                    HALF + HALF * sin(phase * 6.2831853));
    return LIGHTBALL_OUTER_RADIUS * mul;
}

float pulse_radius_default(float time){
    //float pos_in_bar  = mod(time, seconds_per_bar);          // 0 … seconds_per_bar
    //float beat_f      = pos_in_bar / seconds_per_beat;    // 0 … time_signature.x
    //float beat_idx    = floor(beat_f);                    // 0,1,2,3,…
    //float beat_phase  = fract(beat_f);                    // 0 → 1 inside this beat
    //const float R_NEUTRAL = LIGHTBALL_OUTER_RADIUS;   // 0.40  ← baseline
    //const float R_BIG     = 0.50;                     // expand on down-beat
    //const float R_SMALL   = 0.30;                     // shrink on other beats
    //float target = (beat_idx == 0.0) ? R_BIG : R_SMALL;
    //return mix(target, R_NEUTRAL, beat_phase);        // beat_phase: 0 → target, 1 → neutral
    return 0.0;
}

uniform int   beat_index;        // 0,1,2,3…
uniform float beat_phase;        // 0 → 1 during this beat
uniform float seconds_per_beat;  // from TempoDimension
uniform float seconds_per_bar;   // not used below but available
float pulse_radius_true_tempo() {
    const float R_NEUTRAL = LIGHTBALL_OUTER_RADIUS; // 0.40
    const float R_BIG     = 0.50;  // down-beat expand
    const float R_SMALL   = 0.30;  // up-beat shrink

    float target = (beat_index == 0) ? R_BIG : R_SMALL;
    return mix(target, R_NEUTRAL, beat_phase);
}

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
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
    float time = TIME;
    vec2 uv = frag_coord.xy / vec2(iResolution.y);
    vec2 grid_coords = uv_to_grid_space(uv, time);
    //float radius = LIGHTBALL_OUTER_RADIUS;
    //float radius = pulse_radius_experimental(time);
    //float radius = pulse_radius_default(time);
    float radius = pulse_radius_true_tempo();
    vec4 lightball = light_radial_fade(
            grid_coords,
            LIGHTBALL_CENTER,
            radius,
            LIGHTBALL_FADE_BAND
    );
    vec4 src_color = add_dither(lightball, frag_coord);
    COLOR = max(COLOR, src_color);
    COLOR.a = 1.0;
}
