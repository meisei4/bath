extends Node
#class_name MusicDimensionsManager

var spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance

# TODO: where tf does this go, not good,
# do not run rhythmdimension and manual onset recorder at the same time ever
var song_time: float = 0.0

var audio_stream: AudioStream = preload(ResourcePaths.SHADERTOY_MUSIC_EXPERIMENT_OGG)
#var audio_stream: AudioStream = preload(ResourcePaths.HELLION)
#var audio_stream: AudioStream = preload(ResourcePaths.SNUFFY)
#var input_stream: AudioStreamMicrophone = AudioStreamMicrophone.new()


func _ready() -> void:
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)

    #TODO: figure out where to actually have music play in the game, currently
    # juggling too many locations where music is tested for playback
    # especially PitchDimension scene with all the caching and shit

    #AudioPoolManager.play_music(audio_stream)

    #AudioPoolManager.play_input(input_stream)
