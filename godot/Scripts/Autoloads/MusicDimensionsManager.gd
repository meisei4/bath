extends Node
#class_name MusicDimensionsManager

signal onset_detected(current_playback_time: float)

const PERCUSSIVE_FREQUENCY_LOWER_BOUND_HZ: float = 20.0
const PERCUSSIVE_FREQUENCY_UPPER_BOUND_HZ: float = 150.0
const FLUX_HISTORY_BUFFER_MAXIMUM_SIZE: int = 43
const FLUX_THRESHOLD_RATIO: float = 1.5
const MIN_ONSET_INTERVAL: float = 0.25
const ONSETS_PER_MINUTE_HISTORY_BUFFER_MAXIMUM_SIZE: int = 8
const SECONDS_PER_MINUTE: float = 60.0

var spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance
var previous_smooth_energy: float = 0.0
var flux_novelty_curve: PackedFloat32Array = PackedFloat32Array()
var onset_intervals_history_buffer: PackedFloat32Array = PackedFloat32Array()
var time_of_previous_onset: float = 0.0
var onset_event_counter: int = 0

# TODO: where tf does this go, not good,
# do not run rhythmdimension and manual onset recorder at the same time ever
var song_time: float = 0.0

var audio_stream: AudioStream = preload(ResourcePaths.SHADERTOY_MUSIC_EXPERIMENT_OGG)
#var audio_stream: AudioStream = preload(ResourcePaths.HELLION)
#var audio_stream: AudioStream = preload(ResourcePaths.SNUFFY)
#var input_stream: AudioStreamMicrophone = AudioStreamMicrophone.new()


func _enter_tree() -> void:
    print("IN ENTER TREE: mix rate = ", AudioServer.get_mix_rate())


func _ready() -> void:
    print("IN READY: mix rate = ", AudioServer.get_mix_rate())
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)

    #TODO: figure out where to actually have music play in the game, currently
    # juggling too many locations where music is tested for playback
    # especially PitchDimension scene with all the caching and shit

    AudioPoolManager.play_music(audio_stream)

    #AudioPoolManager.play_input(input_stream)
