extends Node
class_name FFTTexture

var audio_texture: ImageTexture
var audio_image: Image
var fft_audio_effect_spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance

var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.MUSIC
#var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.INPUT

var fft_data: PackedFloat32Array

const TEXTURE_HEIGHT: int = 1
const BUFFER_SIZE: int = 512

var SAMPLE_RATE: float = AudioServer.get_mix_rate()  # query real mix rate

const FFT_ROW: int = 0
const DEAD_CHANNEL: float = 0.0

const MDN_MIN_AUDIO_DECIBEL: float = -100.0  #match WebAudio defaults
const MDN_MAX_AUDIO_DECIBEL: float = -30.0  #match WebAudio defaults
const MDN_BINS: int = 1024
const MDN_SMOOTHING: float = 0.8


func _ready() -> void:
    prepare_fft_audio_effect_spectrum_analyzer()
    audio_image = Image.create(BUFFER_SIZE, TEXTURE_HEIGHT, false, Image.FORMAT_R8)
    audio_texture = ImageTexture.create_from_image(audio_image)


func _process(delta: float) -> void:
    update_fft_texture_row()
    audio_texture.set_image(audio_image)


func prepare_fft_audio_effect_spectrum_analyzer() -> void:
    var fft_audio_effect_spectrum_analyzer: AudioEffectSpectrumAnalyzer = (
        AudioEffectSpectrumAnalyzer.new()
    )
    var audio_bus_index: int = AudioBus.get_bus_index(TARGET_AUDIO_BUS)
    var effect_idx: int = AudioEffects.add_effect(
        TARGET_AUDIO_BUS, fft_audio_effect_spectrum_analyzer
    )
    fft_audio_effect_spectrum_analyzer_instance = (
        AudioServer.get_bus_effect_instance(audio_bus_index, effect_idx)
        as AudioEffectSpectrumAnalyzerInstance
    )
    fft_audio_effect_spectrum_analyzer.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    #fft_audio_effect_spectrum_analyzer.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_1024
    fft_data.resize(BUFFER_SIZE)


func update_fft_texture_row() -> void:
    for bin_index: int in range(BUFFER_SIZE):
        var from_hz: float = bin_index * (SAMPLE_RATE * 0.5) / MDN_BINS
        var to_hz: float = (bin_index + 1) * (SAMPLE_RATE * 0.5) / MDN_BINS
        var stereo_magnitude: Vector2 = (
            fft_audio_effect_spectrum_analyzer_instance
            . get_magnitude_for_frequency_range(
                from_hz, to_hz, AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
            )
        )
        var downmix_stereo_magnitude: float = (stereo_magnitude.x + stereo_magnitude.y) * 0.5
        var db: float = linear_to_db(downmix_stereo_magnitude)
        var raw_norm: float = clamp(
            (db - MDN_MIN_AUDIO_DECIBEL) / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL), 0.0, 1.0
        )
        fft_data[bin_index] = MDN_SMOOTHING * fft_data[bin_index] + (1.0 - MDN_SMOOTHING) * raw_norm
        var norm: float = fft_data[bin_index]  # single-stage quant happens when Godot packs R8
        audio_image.set_pixel(
            bin_index, FFT_ROW, Color(norm, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
        )
