shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/Audio/utils.gdshaderinc"

uniform vec2 iResolution;
uniform sampler2D iChannel0: hint_screen_texture;
uniform sampler2D iChannel1; //audio texture

#define WAVEFORM_SAMPLE_COUNT 512 //TODO: derive this from the actual ShaderToyAudioTexture constants??? it doesnt even need to 512?
#define WAVEFORM_SAMPLE_COUNT_F float(WAVEFORM_SAMPLE_COUNT)
#define AUDIO_TEXTURE_WAVEFORM_ROW 1.0 //TODO: derive this from the actual ShaderToyAudioTexture constants

#define UNUSED_FEEDBACK_CHANNEL 0.0
#define WAVEFORM_SEGMENT_CENTERING_OFFSET 0.5

#define WAVEFORM_SAMPLES_PER_SEGMENT (WAVEFORM_SAMPLE_COUNT / DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS)
#define WAVEFORM_SAMPLES_PER_SEGMENT_F (WAVEFORM_SAMPLE_COUNT_F / DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS_F)

//TODO: BLEND CYCLE LOOKS KIND OF COOL AS A PSUEDO RHYTHM
#define INJECTION_BLEND_CYCLE_REFRESH_RATE_IN_SECONDS 2.0 //every half second the mix function for interpolation between histories changes
#define TARGET_BLEND_INTENSITY_BETWEEN_ENVELOPE_INJECTION_SNAPSHOTS 0.6 // coefficient at wich the new wavform injection blends with previous waveform injection
//TODO: AHHHHHHHHHHHHHHHHHHHHHHH the idea is to evolve these MACROS OVER THE COURSE OF A SONG BASED ON THE SONGS BEHAVIOR!!!
// TIME (in seconds) between each waveform injection occurance (i.e. how often you want to open the injection window/inject waveforms into the envelope)
#define INJECTION_INTERVAL 0.1 //e.g. every 1.0 second: open the injection window

 // SIZE (percentage of the injection interval) that determines how many buffered waveform samples to inject into the envelope each cycle (i.e.how long to keep the window open each cycle)
#define INJECTION_WINDOW 0.8 // e.g. when injection occurs allow 20% of 1.0 second = 0.2 seconds of buffered waveforms to enter the envelope (TODO: THUS THIS IS directly TIED TO THE AudioEffectCapture.buffer_length

// RATE (percentage of the frame rate/gpu screen pass) of how fast you want to propagate/update the envelope snapshots (historical lines)
#define PROPAGATION_RATE_FRAME 0.5 // coupled to frame rate e.g. for 30fps, every 6 frames (20% of 30fps) = 6/30 fps -> propagation occurs every 0.2 seconds, i.e. ~5 times per second. propagate the injected waveforms upwards through the envelope snapshots

float compute_normalized_sample_coordinate_for_downscaled_segment_in_1D(float segment_index, float sample_index) {
    float sample_coordinate_on_1D_x_axis = segment_index * WAVEFORM_SAMPLES_PER_SEGMENT_F + sample_index + WAVEFORM_SEGMENT_CENTERING_OFFSET;
    return sample_coordinate_on_1D_x_axis / WAVEFORM_SAMPLE_COUNT_F; // normalized audio texture sample coordinate
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

float propagate_envelope_injection_snapshot_upwards(vec2 uv, float y_shift) {
    vec4 envelope_fragment = texture(iChannel0, uv + vec2(0.0, y_shift));
    return envelope_fragment.r;
}

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 uv = frag_coord.xy / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    float uv_height_per_envelope_snapshot = 1.0 / float(NUMBER_OF_HISTORICAL_ENVELOPE_SNAPSHOTS);
    // when screen pass is in the upper section of the uv, target all envelope snapshots above the initial (front most) envelope
    if (uv.y < 1.0 - uv_height_per_envelope_snapshot) {
        float envelope_fragment = propagate_envelope_injection_snapshot_upwards(uv, PROPAGATION_RATE_FRAME * uv_height_per_envelope_snapshot);
        //frag_color = vec4(envelope_fragment, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
        COLOR = vec4(envelope_fragment, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
    } else { // target the bottom most envelope
        float current_envelope = texture(iChannel0, uv).r;
        //float normalized_time_since_last_injection = mod(iTime, INJECTION_INTERVAL) / INJECTION_INTERVAL; //0 = injection cycle just started, 1 = waveform injection window opens and cycle restarts
        //float normalized_time_since_last_injection = mod(TIME, INJECTION_INTERVAL) / INJECTION_INTERVAL; //0 = injection cycle just started, 1 = waveform injection window opens and cycle restarts
        float normalized_time_since_last_injection = fract(TIME / INJECTION_INTERVAL); //0 = injection cycle just started, 1 = waveform injection window opens and cycle restarts
        //if the time passed since last injection lies WITHIN [0, INJECTION_WINDOW), then inject next waveform data into the envelope
        if (normalized_time_since_last_injection < INJECTION_WINDOW) {
            //TODO: the below formula is used to get the current segment index as as discrete space coordinate from the downscaled segment distribution
            // floor is used to prioritize mantissa casting to int to avoid potential out of bounds segment index
            //at segment_index = DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS exactly (because the last segment index is actually DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS_F - 1
            int segment_index_in_discrete_space = int(floor(uv.x * DOWNSCALED_TARGET_NUMBER_OF_WAVEFORM_SEGMENTS_F));
            float next_envelope = sample_audio_texture_waveform_data(segment_index_in_discrete_space);
            //float injection_blend_intensity_coefficient = mod(iTime, INJECTION_BLEND_CYCLE_REFRESH_RATE_IN_SECONDS) / INJECTION_BLEND_CYCLE_REFRESH_RATE_IN_SECONDS;
            //float injection_blend_intensity_coefficient = mod(TIME, INJECTION_BLEND_CYCLE_REFRESH_RATE_IN_SECONDS) / INJECTION_BLEND_CYCLE_REFRESH_RATE_IN_SECONDS;
            float injection_blend_intensity_coefficient = 1.0;
            float t = normalized_time_since_last_injection / INJECTION_WINDOW;

            float smooth_injection = 0.5 * (1.0 - cos(6.28318 * t));
            float effective_injection_blend = smooth_injection * injection_blend_intensity_coefficient * TARGET_BLEND_INTENSITY_BETWEEN_ENVELOPE_INJECTION_SNAPSHOTS;
            float blended_envelope = mix(current_envelope, next_envelope, effective_injection_blend);
            //frag_color = vec4(blended_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
            COLOR = vec4(blended_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
        } else {
            //frag_color = vec4(current_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
            COLOR = vec4(current_envelope, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL, UNUSED_FEEDBACK_CHANNEL);
        }
    }
}
