shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/Audio/SoundEnvelopeWIP/utils.gdshaderinc"

uniform vec2 iResolution;
uniform sampler2D iChannel0: hint_screen_texture;
uniform sampler2D iChannel1; //audio texture

#define WAVEFORM_SAMPLE_COUNT 512 //TODO: derive this from the actual ShaderToyAudioTexture constants??? it doesnt even need to 512?
#define WAVEFORM_SAMPLE_COUNT_F float(WAVEFORM_SAMPLE_COUNT)
#define AUDIO_TEXTURE_WAVEFORM_ROW 1.0 //TODO: derive this from the actual ShaderToyAudioTexture constants

#define WAVEFORM_SEGMENT_CENTERING_OFFSET 0.5

#define WAVEFORM_SAMPLES_PER_SEGMENT (WAVEFORM_SAMPLE_COUNT / DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS)
#define WAVEFORM_SAMPLES_PER_SEGMENT_F (WAVEFORM_SAMPLE_COUNT_F / DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS_F)

#define TARGET_BLEND_INTENSITY_BETWEEN_ENVELOPE_INJECTION_SNAPSHOTS 0.5
#define INJECTION_INTERVAL 0.25 //e.g. every 0.1 second: open the injection window
#define INJECTION_WINDOW 0.95 // e.g. when injection occurs allow 80% of 0.1 second = 0.08 seconds of buffered waveforms to enter the envelope???

float compute_normalized_sample_coordinate_for_downscaled_segment_in_1D(float segment_index, float sample_index) {
    float sample_coordinate_on_1D_x_axis = segment_index * WAVEFORM_SAMPLES_PER_SEGMENT_F + sample_index + WAVEFORM_SEGMENT_CENTERING_OFFSET;
    return sample_coordinate_on_1D_x_axis / WAVEFORM_SAMPLE_COUNT_F;
}

float sample_audio_texture_waveform_data(int segment_index) {
    float accumulated_amplitude = 0.0;
    for (int sample_index = 0; sample_index < WAVEFORM_SAMPLES_PER_SEGMENT; sample_index++) {
        float segment_index_in_continious_space = float(segment_index);
        float sample_index_in_continious_space = float(sample_index);
        float normalized_sample_coordinate_for_downscaled_segment_in_1D = compute_normalized_sample_coordinate_for_downscaled_segment_in_1D(segment_index_in_continious_space, sample_index_in_continious_space);
        vec2 sample_coordinates = vec2(normalized_sample_coordinate_for_downscaled_segment_in_1D, AUDIO_TEXTURE_WAVEFORM_ROW);
        float amplitude_value_at_sample_coordinates = texture(iChannel1, sample_coordinates).r;
        accumulated_amplitude += abs(amplitude_value_at_sample_coordinates);
    }
    return accumulated_amplitude / WAVEFORM_SAMPLES_PER_SEGMENT_F;
}


//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment(){
    float u = FRAGCOORD.x / iResolution.x;
    // Force vertical coordinate to 0.0 (for instance, the first row). BUFFER A IS NOW 1D!!
    float v = 0.0;
    vec2 uv = vec2(u, v);
    float current_envelope = texture(iChannel0, uv).r;
    float normalized_time_since_last_injection = fract(TIME / INJECTION_INTERVAL);
    if (normalized_time_since_last_injection < INJECTION_WINDOW) {
        int segment_index_in_discrete_space = int(floor(uv.x * DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS_F));
        float next_envelope = sample_audio_texture_waveform_data(segment_index_in_discrete_space);
        float injection_blend_intensity_coefficient = 1.0;
        float t = normalized_time_since_last_injection / INJECTION_WINDOW;

        float smooth_injection = 0.5 * (1.0 - cos(6.28318 * t)); //TODO: I dont like this, but it kind of helps with smoothing?
        float effective_injection_blend = smooth_injection * injection_blend_intensity_coefficient * TARGET_BLEND_INTENSITY_BETWEEN_ENVELOPE_INJECTION_SNAPSHOTS;
        float blended_envelope = mix(current_envelope, next_envelope, effective_injection_blend);
        COLOR = vec4(blended_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
    } else {
        COLOR = vec4(current_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
    }
}
