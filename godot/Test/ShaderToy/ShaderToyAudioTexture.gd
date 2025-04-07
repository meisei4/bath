extends Node
class_name ShaderToyAudioTexture

var audio_texture: ImageTexture
var audio_image: Image
var waveform_audio_effect_capture: AudioEffectCapture
var fft_audio_effect_spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance

var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.MUSIC
var MAGNITUDE_MODE: AudioEffectSpectrumAnalyzerInstance.MagnitudeMode = (
    AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
)

var waveform_data: Array[float]  #TODO: use these as uniforms perhaps later if it improves performance
var fft_data: Array[float]  #TODO: use these as uniforms perhaps later if it improves performance
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
const AUDIO_DECIBEL_FLOOR: float = -100.0  # some default "quietest" decibel level (near silence) in the audio.
const AUDIO_DECIBEL_CIELING: float = 0.0  #the loudest?? is it just from the AudioBus attribute???

const WAVEFORM_ROW: int = 1
const FFT_ROW: int = 0
const DEAD_CHANNEL: float = 0.0


func _ready() -> void:
    #TODO: figure out how to get the AudioEffectSpectrumAnalyzerInstance and AudioEffectCapture to be active at the same time (separate buses??)
    #prepare_fft_audio_effect_spectrum_analyzer()
    prepare_waveform_audio_effect_capture()

    #TODO: make the image only 32-bit Red channel only: Image.FORMAT_RF = Red channel Full 32 bit range
    audio_image = Image.create(BUFFER_SIZE, TEXTURE_HEIGHT, false, Image.FORMAT_RF)
    audio_texture = ImageTexture.create_from_image(audio_image)


func _process(_delta: float) -> void:
    #update_fft_texture_row()
    update_waveform_texture_row()
    audio_texture.set_image(audio_image)


func prepare_waveform_audio_effect_capture() -> void:
    waveform_audio_effect_capture = AudioEffectCapture.new()
    #waveform_audio_effect_capture.buffer_length =  0.01666 # 1/60??
    waveform_audio_effect_capture.buffer_length = 0.03333333333  #TODO how to get a proper framerate based buffer length or is it even what i want? it looks decent like this tbh........
    AudioEffects.add_effect(TARGET_AUDIO_BUS, waveform_audio_effect_capture)
    waveform_data.resize(BUFFER_SIZE)


func update_waveform_texture_row() -> void:
    if waveform_audio_effect_capture.can_get_buffer(BUFFER_SIZE):
        #TODO: the get_buffer may return a variable sized vector array, since its just how many could be put into the buffer
        # get_buffer behavior: The samples are signed floating-point PCM between -1 and 1.
        # You will have to scale them if you want to use them as 8 or 16-bit integer samples:
        # (v = 0x7fff * samples[0].x)
        #TODO: experiment more with how to truly control this buffer size i the audio sampling
        var captured_frames_from_current_waveform_buffer: PackedVector2Array = (
            waveform_audio_effect_capture.get_buffer(BUFFER_SIZE)
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
            var audio_texture_value: Color = Color(
                average_amplitude, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL
            )
            audio_image.set_pixel(x, WAVEFORM_ROW, audio_texture_value)
    else:
        for x: int in range(BUFFER_SIZE):
            var audio_texture_value: Color = Color(
                waveform_data[x], DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL
            )
            audio_image.set_pixel(x, WAVEFORM_ROW, audio_texture_value)


func prepare_fft_audio_effect_spectrum_analyzer() -> void:
    var fft_audio_effect_spectrum_analyzer: AudioEffectSpectrumAnalyzer = (
        AudioEffectSpectrumAnalyzer.new()
    )
    fft_audio_effect_spectrum_analyzer.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_512
    AudioEffects.add_effect(TARGET_AUDIO_BUS, fft_audio_effect_spectrum_analyzer)
    var audio_bus_index: int = AudioBus.get_bus_index(TARGET_AUDIO_BUS)
    fft_audio_effect_spectrum_analyzer_instance = (
        AudioServer.get_bus_effect_instance(audio_bus_index, 0)
        as AudioEffectSpectrumAnalyzerInstance
    )
    fft_data.resize(BUFFER_SIZE)


#TODO: this is not actually achieving the same FFT derivation that shadertoy web uses, but VSCode extension is also wacked up, so just derive it later custom or something
func update_fft_texture_row() -> void:
    #first row is frequency data: 48Khz / 4 = 12 kHz
    var total_frequency_range_sample_target_scaled_down: float = SAMPLE_RATE / 4.0
    # in 512 texels, meaning 23 Hz per texel, i.e. (48Khz / 4)  / 512 = 12 kHz /512 bins
    var width_in_frequencies_per_bin: float = (
        total_frequency_range_sample_target_scaled_down / float(BUFFER_SIZE)
    )
    # i.e. ~23.4 Hz wide, bin_0 covers 0–23.4 Hz, bin_1 covers 23.4–46.8 Hz, etc.
    var prev_hz: float = 0.0
    for x: int in range(BUFFER_SIZE):
        var current_hz: float = (x + 1) * width_in_frequencies_per_bin
        var current_frequency_bin_amplitude_left_channel: float = (
            fft_audio_effect_spectrum_analyzer_instance
            . get_magnitude_for_frequency_range(prev_hz, current_hz, MAGNITUDE_MODE)
            . x
        )
        var linear_amplitude_scaled_to_decibels: float = (
            20.0 * log10(current_frequency_bin_amplitude_left_channel + 1e-12)
        )
        var db_norm: float = (
            (linear_amplitude_scaled_to_decibels - AUDIO_DECIBEL_FLOOR)
            / (AUDIO_DECIBEL_CIELING - AUDIO_DECIBEL_FLOOR)
        )
        var db_clamped: float = clamp(db_norm, 0.0, 1.0)  #TODO anything below -100 dB becomes 0, anything greater than 0 dB becomes 1
        var audio_texture_value: Color = Color(db_clamped, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
        audio_image.set_pixel(x, FFT_ROW, audio_texture_value)
        prev_hz = current_hz


func log10(value: float) -> float:
    return log(value) / log(10.0)
