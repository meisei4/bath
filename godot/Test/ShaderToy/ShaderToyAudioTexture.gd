extends Node
class_name ShaderToyAudioTexture

var audio_texture: ImageTexture
var audio_image: Image
var fft_spectrum: AudioEffectSpectrumAnalyzerInstance
var waveform_data_capture: AudioEffectCapture

var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.MUSIC
var MAX_MAGNITUDE_MODE_LOL: AudioEffectSpectrumAnalyzerInstance.MagnitudeMode

var fft_data: Array[float]  #TODO: use these as uniforms perhaps later if it improves performance
var waveform_data: Array[float]  #TODO: use these as uniforms perhaps later if it improves performance
# When sampling a texture, you typically sample at the center of a texel.
# For a texture that’s 2 pixels high, the centers of the two rows are calculated as:
#Top row (y = 0): Center is at (0 + 0.5) / 2 = 0.25
#Bottom row (y = 1): Center is at (1 + 0.5) / 2 = 0.75
#Thus, in ShaderToy:
#Sampling at y = 0.25 fetches the FFT data.
#Sampling at y = 0.75 fetches the raw waveform data.
const TEXTURE_HEIGHT: int = 2  #y = 0 is fft spectrum, y= 1 is raw wave data
const BUFFER_SIZE: int = 512
const SAMPLE_RATE: float = 48000.0  #TODO: figure out how to get this to actually be shadertoy matched
const TOTAL_RANGE_HZ: float = 12000.0 #for exactly 12 kHz coverage over 512 bins
const FFT_ROW: int = 0
const WAVEFORM_ROW: int = 1
const DEAD_CHANNEL: float = 0.0

func _ready() -> void:
    var fft_effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    fft_effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_512
    AudioEffects.add_effect(TARGET_AUDIO_BUS, fft_effect)
    var audio_bus_index: int = AudioBus.get_bus_index(TARGET_AUDIO_BUS)
    fft_spectrum = (
        AudioServer.get_bus_effect_instance(audio_bus_index, 0)
        as AudioEffectSpectrumAnalyzerInstance
    )
    #TODO: make the image only 32-bit Red channel only: Image.FORMAT_RF = Red channel Full 32 bit range
    audio_image = Image.create(BUFFER_SIZE, TEXTURE_HEIGHT, false, Image.FORMAT_RF)
    audio_texture = ImageTexture.create_from_image(audio_image)

    fft_data.resize(BUFFER_SIZE)

    #waveform_data_capture = AudioEffectCapture.new()
    #waveform_data_capture.buffer_length = 0.1 #????? 10th of a second?
    #AudioEffects._add_effect(TARGET_AUDIO_BUS, waveform_data_capture)
    waveform_data.resize(BUFFER_SIZE)


func _process(_delta: float) -> void:
    update_fft_texture_channel()
    #update_waveform_texture_channel()
    audio_texture.set_image(audio_image)


func update_fft_texture_channel() -> void:
    MAX_MAGNITUDE_MODE_LOL = AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
    #var width_per_frequency_bin: float = TOTAL_RANGE_HZ / float(BUFFER_SIZE)
    var width_per_frequency_bin: float = (SAMPLE_RATE / 4.0) / float(BUFFER_SIZE)
    var prev_hz: float = 0.0
    for x: int in range(BUFFER_SIZE):
        var current_hz: float = (x + 1) * width_per_frequency_bin
        var current_frequency_bin_amplitude_left_channel: float = (
            fft_spectrum
            . get_magnitude_for_frequency_range(prev_hz, current_hz, MAX_MAGNITUDE_MODE_LOL)
            . x
        )
        # human hearing range decibels
        var db: float = 20.0 * log10(current_frequency_bin_amplitude_left_channel + 1e-12)
        var min_db: float = -60.0   # floor
        var max_db: float = 0.0     # ceiling
        var db_norm: float = (db - min_db) / (max_db - min_db) #normal range for
        db_norm = clamp(db_norm, 0.0, 1.0) #TODO WHYYY
        var audio_texture_value: Color = Color(db_norm, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
        audio_image.set_pixel(x, FFT_ROW, audio_texture_value)
        prev_hz = current_hz


func log10(value: float) -> float:
    return log(value) / log(10.0)


func update_waveform_texture_channel() -> void:
    if waveform_data_capture.can_get_buffer(BUFFER_SIZE):
        #TODO: the get_buffer may return a variable sized vector array, since its just how many could be put into the buffer
        # get_buffer behavior: The samples are signed floating-point PCM between -1 and 1.
        # You will have to scale them if you want to use them as 8 or 16-bit integer samples:
        # (v = 0x7fff * samples[0].x)
        #TODO: experiment more with how to truly control this buffer size i the audio sampling
        var captured_frames_from_current_waveform_buffer: PackedVector2Array = (
            waveform_data_capture.get_buffer(BUFFER_SIZE)
        )
        var frame_count: float = captured_frames_from_current_waveform_buffer.size()
        #TODO: we need to scale down the buffer to fit into our 512 pixel length texture (ji.e. divide whatever size comes out of the capture)
        var frames_per_pixel: float = frame_count / BUFFER_SIZE
        for x: int in range(BUFFER_SIZE):
            var start_frame_index: float = x * frames_per_pixel
            var end_frame_index: float = (x + 1) * frames_per_pixel
            if end_frame_index > frame_count:
                end_frame_index = frame_count
            var accumulated_amplitudes: float = 0.0
            var number_of_amplitude_frames_to_average: int = 0
            for i: int in range(start_frame_index, end_frame_index):
                accumulated_amplitudes += captured_frames_from_current_waveform_buffer[i].x  #TODO: assumes mono audio PLEASE! (only get x, because x=y for mono)
                number_of_amplitude_frames_to_average += 1
            var average_amplitude: float = (
                accumulated_amplitudes / number_of_amplitude_frames_to_average
                if number_of_amplitude_frames_to_average > 0
                else 0.0
            )
            # TODO: normalize the amplitudes for the waveform data to be [0,1] (not -1,1)
            # optional accumulated_amplitudes += 0x7fff * captured_frames_from_current_waveform_buffer[0].x) to get 8bit or 16bit???
            average_amplitude = average_amplitude * 0.5 + 0.5
            waveform_data[x] = average_amplitude
            var audio_texture_value: Color = Color(average_amplitude, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
            audio_image.set_pixel(x, WAVEFORM_ROW, audio_texture_value)
    else:
        for x: int in range(BUFFER_SIZE):
            var audio_texture_value: Color = Color(waveform_data[x], DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
            audio_image.set_pixel(x, WAVEFORM_ROW, audio_texture_value)
