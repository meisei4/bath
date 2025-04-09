shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/Audio/utils.gdshaderinc"

uniform vec2 iResolution;
uniform int iFrame;
uniform float iTime;
uniform sampler2D iChannel0: hint_screen_texture;
uniform sampler2D iChannel1; //audio texture

#define WAVEFORM_SAMPLE_COUNT 512
#define UNUSED_FEEDBACK_CHANNEL 0.0

//TODO: BLEND CYCLE LOOKS KIND OF COOL AS A PSUEDO RHYTHM
const float BLEND_CYCLE_REFRESH_RATE_IN_SECONDS = 2.0; //every half second the mix function for interpolation between histories changes
const float MAX_INTERPOLATION_INTENSITY_BETWEEN_WAVEFORM_HISTORY_SNAPSHOTS = 0.3; // coefficient at wich the new wavform injection blends with previous waveform injection

const float INJECTION_INTERVAL = 0.40; // cycle time in seconds kmjtrfor injecting current waveform into the front row
// TODO: THIS IS SCARY, some how this is a window that serves as a "normalized" percentage of the INJECTION_INTERVAL
// (but it can result in instability if the engine's iTime or gpu's screen pass skips over this range?
// it might also be whats causing the random SPEED UPS after the shader runs for a bit
const float INJECTION_WINDOW = 0.5;
const float PROPAGATION_RATE = 0.5; // HOW SLOW YOU WANT THE PROPAGATION BETWEEN WAVFORM HISTORY BANDS GO (1.0 is max speed?)

const int samples_per_bin = WAVEFORM_SAMPLE_COUNT / NUMBER_OF_BINS;

float sample_waveform_envelope(int bin_index) {
    float sum_amplitude = 0.0;
    for (int i = 0; i < samples_per_bin; i++) {
        float sample_x = (float(bin_index * samples_per_bin + i) + 0.5) / float(WAVEFORM_SAMPLE_COUNT);
        float sample_value = texture(iChannel1, vec2(sample_x, 1.0)).r;
        sum_amplitude += abs(sample_value);
    }
    return sum_amplitude / float(samples_per_bin);
}

float shift_envelope_history_fragment_vertically(vec2 uv, float y_shift) {
    vec4 old_fragment = texture(iChannel0, uv + vec2(0.0, y_shift));
    return old_fragment.r;
}

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 uv = frag_coord.xy / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    //float normalized_time_since_last_injection = mod(iTime, INJECTION_INTERVAL) / INJECTION_INTERVAL; //0= propagation cycle just started, 1= propagation cycle should end
    float normalized_time_since_last_injection = mod(TIME, INJECTION_INTERVAL) / INJECTION_INTERVAL; //0= propagation cycle just started, 1= propagation cycle should end
    //TODO: math wtf????
    //float cycle_time = TIME / INJECTION_INTERVAL;
    //Extract the fractional part to get the current cycle phase (from 0.0 to 1.0)
    //float cycle_phase = fract(cycle_time);
    float row_height = 1.0 / float(NUMBER_OF_HISTORY_ROWS);
    if (uv.y < 1.0 - row_height ) {
        float old_fragment_red_channel = shift_envelope_history_fragment_vertically(uv, PROPAGATION_RATE * row_height);
        //frag_color = vec4(old_fragment_red_channel, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
        COLOR = vec4(old_fragment_red_channel, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
    } else {
        //float interpolation_intensity_coefficient = mod(iTime, BLEND_CYCLE_REFRESH_RATE_IN_SECONDS) / BLEND_CYCLE_REFRESH_RATE_IN_SECONDS;
        //float interpolation_intensity_coefficient = mod(TIME, BLEND_CYCLE_REFRESH_RATE_IN_SECONDS) / BLEND_CYCLE_REFRESH_RATE_IN_SECONDS;
        float interpolation_intensity_coefficient = 1.0;
        int bin_index = int(floor(uv.x * float(NUMBER_OF_BINS)));
        float new_envelope = sample_waveform_envelope(bin_index);
        float old_envelope = texture(iChannel0, uv).r;
        //if (cycle_phase >= INJECTION_WINDOW) {
        if (normalized_time_since_last_injection >= INJECTION_WINDOW) { //when propagation "timer" passes the injection window, inject new waveform data
            //frag_color = vec4(old_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
            COLOR = vec4(old_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
        } else {
            float effective_blend = interpolation_intensity_coefficient * MAX_INTERPOLATION_INTENSITY_BETWEEN_WAVEFORM_HISTORY_SNAPSHOTS;
            float blended_envelope = mix(old_envelope, new_envelope, effective_blend);
            //frag_color = vec4(blended_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
            COLOR = vec4(blended_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
        }
    }
}
