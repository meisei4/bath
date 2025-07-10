extends Node
#class_name MusicDimensionsManager

var spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance
var song_time: float = 0.0 # TODO: where tf does this go, not good,
var audio_stream: AudioStream = preload(ResourcePaths.SHADERTOY_MUSIC_EXPERIMENT_OGG)
#var input_stream: AudioStreamMicrophone = AudioStreamMicrophone.new()


func _ready() -> void:
    var bus_index: int = AudioBus.get_bus_index(AudioBus.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)

    #AudioPoolManager.play_music(audio_stream)
    #AudioPoolManager.play_input(input_stream, 0.0)
