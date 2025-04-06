extends Node
class_name ShaderToyAudioTexture

var audio_texture: ImageTexture
var audio_image: Image
var fft_spectrum: AudioEffectSpectrumAnalyzerInstance
var waveform_data_capture: AudioEffectCapture

var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.MUSIC
var MAX_MAGNITUDE_MODE_LOL: AudioEffectSpectrumAnalyzerInstance.MagnitudeMode

var buffer_size = 512
# When sampling a texture, you typically sample at the center of a texel.
# For a texture that’s 2 pixels high, the centers of the two rows are calculated as:
#Top row (y = 0): Center is at (0 + 0.5) / 2 = 0.25
#Bottom row (y = 1): Center is at (1 + 0.5) / 2 = 0.75
#Thus, in ShaderToy:
#Sampling at y = 0.25 fetches the FFT data.
#Sampling at y = 0.75 fetches the raw waveform data.
var texture_height = 2  #y = 0 is fft spectrum, y= 1 is raw wave data
var fft_data: Array[float]  #TODO: use these as uniforms perhaps later if it improves performance
var waveform_data: Array[float]  #TODO: use these as uniforms perhaps later if it improves performance

var sample_rate: float = 44100.0  #TODO: figure out how to get this to actually be shadertoy matched
var frequency_bin_size: float
var width_per_frequency_bin: float

const DEAD_CHANNEL: float = 0.0

const FFT_ROW: float = 0.25
const WAVEFORM_ROW: float = 0.75


func _ready() -> void:
    var fft_effect = AudioEffectSpectrumAnalyzer.new()
    fft_effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_512
    AudioEffects._add_effect(TARGET_AUDIO_BUS, fft_effect)

    var audio_bus_string_name: StringName = AudioBus.val(TARGET_AUDIO_BUS)
    var audio_bus_index = AudioServer.get_bus_index(audio_bus_string_name)
    fft_spectrum = (
        AudioServer.get_bus_effect_instance(audio_bus_index, 0)
        as AudioEffectSpectrumAnalyzerInstance
    )

    frequency_bin_size = sample_rate / 4.0
    width_per_frequency_bin = frequency_bin_size / buffer_size

    #waveform_data_capture = AudioEffectCapture.new()
    #waveform_data_capture.buffer_length = 0.1 #????? 10th of a second?
    #AudioEffects._add_effect(TARGET_AUDIO_BUS, waveform_data_capture)

    #TODO: make the image only 32-bit Red channel only: Image.FORMAT_RF = Red channel Full 32 bit range
    audio_image = Image.create(buffer_size, texture_height, false, Image.FORMAT_RF)
    audio_texture = ImageTexture.create_from_image(audio_image)

    fft_data.resize(buffer_size)
    waveform_data.resize(buffer_size)


func _process(_delta) -> void:
    update_fft_texture_channel()
    #update_waveform_texture_channel()
    audio_texture.set_image(audio_image)


#TODO: something is going terribly wrong here that doesnt result in the same scaling as shadertoys FFT channel
func update_fft_texture_channel() -> void:
    MAX_MAGNITUDE_MODE_LOL = AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
    var prev_hz: float = 0.0
    for x: int in range(buffer_size):
        var current_hz: float = (x + 1) * width_per_frequency_bin
        var freq_bin_magnitude_left_channel: float = (
            fft_spectrum
            . get_magnitude_for_frequency_range(
                prev_hz,
                current_hz,
                MAX_MAGNITUDE_MODE_LOL
            )
            . x
        )
        var db = 20.0 * log10(freq_bin_magnitude_left_channel)
        var db_scaled = (db + 60.0) / 60.0  # TODO: i dont even know if shadertoy is using db scaling no idea
        db_scaled = clamp(db_scaled, 0.0, 1.0)
        audio_image.set_pixel(
            x, FFT_ROW, Color(db_scaled, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
        )
        prev_hz = current_hz


func log10(value: float) -> float:
    return log(value) / log(10.0)


func update_waveform_texture_channel():
    if waveform_data_capture.can_get_buffer(buffer_size):
        #TODO: the get_buffer may return a variable sized vector array, since its just how many could be put into the buffer
        # get_buffer behavior: The samples are signed floating-point PCM between -1 and 1.
        # You will have to scale them if you want to use them as 8 or 16-bit integer samples:
        # (v = 0x7fff * samples[0].x)
        #TODO: experiment more with how to truly control this buffer size i the audio sampling
        var captured_frames_from_current_waveform_buffer: PackedVector2Array = (
            waveform_data_capture.get_buffer(buffer_size)
        )
        var frame_count = captured_frames_from_current_waveform_buffer.size()
        #TODO: we need to scale down the buffer to fit into our 512 pixel length texture (ji.e. divide whatever size comes out of the capture)
        var frames_per_pixel = frame_count / buffer_size
        for x: int in range(buffer_size):
            var start_frame_index = int(x * frames_per_pixel)
            var end_frame_index = int((x + 1) * frames_per_pixel)
            if end_frame_index > frame_count:
                end_frame_index = frame_count
            var accumulated_amplitudes: float = 0.0
            var number_of_amplitude_frames_to_average: int = 0
            for i: int in range(start_frame_index, end_frame_index):
                accumulated_amplitudes += captured_frames_from_current_waveform_buffer[i].x  #TODO: assumes mono audio PLEASE! (only get x, because x=y for mono)
                number_of_amplitude_frames_to_average += 1
            var average_amplitude = (
                accumulated_amplitudes / number_of_amplitude_frames_to_average
                if number_of_amplitude_frames_to_average > 0
                else 0.0
            )
            # TODO: normalize the amplitudes for the waveform data to be [0,1] (not -1,1)
            # optional accumulated_amplitudes += 0x7fff * captured_frames_from_current_waveform_buffer[0].x) to get 8bit or 16bit???
            average_amplitude = average_amplitude * 0.5 + 0.5
            waveform_data[x] = average_amplitude
            audio_image.set_pixel(
                x, WAVEFORM_ROW, Color(waveform_data[x], DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
            )
    else:
        for x: int in range(buffer_size):
            audio_image.set_pixel(
                x, WAVEFORM_ROW, Color(waveform_data[x], DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
            )
