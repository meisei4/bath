shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/Audio/audio_plotting.gdshaderinc"

uniform vec2 iResolution;
uniform int iFrame;
uniform float iTime;
uniform sampler2D iChannel0: hint_screen_texture;
uniform sampler2D iChannel1; //audio texture

#define WAVEFORM_SAMPLE_COUNT 512

//TODO: turn these into uniforms maybe 
const float uUpdateInterval = 11.0;    // e.g. 15.0 for more frequent updates.
const float uEnvelopeSmoothing = 0.2;    // e.g. 0.3 to 0.5 for moderate smoothing.

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

vec4 shift_envelope_history(vec2 uv, float row_height) {
    return texture(iChannel0, uv + vec2(0.0, row_height));
}

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 uv = frag_coord.xy / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;

    float row_height = 1.0 / float(NUMBER_OF_HISTORY_ROWS);
    if (uv.y < 1.0 - row_height) {
        //frag_color = shift_envelope_history(uv, row_height);
        COLOR = shift_envelope_history(uv, row_height);
    } else {
        float update_interval = uUpdateInterval; 
        float blend_factor = mod(float(iFrame), update_interval) / update_interval;
        int bin_index = int(floor(uv.x * float(NUMBER_OF_BINS)));
        float new_envelope = sample_waveform_envelope(bin_index);
        float old_envelope = texture(iChannel0, uv).r;
        
        float effective_blend = blend_factor * uEnvelopeSmoothing;
        float blended_envelope = mix(old_envelope, new_envelope, effective_blend);
        //frag_color = vec4(blended_envelope, blended_envelope, blended_envelope, 1.0);
        COLOR = vec4(blended_envelope, blended_envelope, blended_envelope, 1.0);
    }
}
