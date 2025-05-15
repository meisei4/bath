extends Node
#class_name MusicDimensionsManager

signal beat_detected(beat_index: int, time_since_last_beat_in_seconds: float, bpm: float)

var SAMPLE_RATE: float = AudioServer.get_mix_rate()  # query real mix rate

const FREQUENCY_LOWER_BOUND_HERTZ: float = 20.0
const FREQUENCY_UPPER_BOUND_HERTZ: float = 150.0
const ABSOLUTE_FLOOR_LEVEL: float = 0.01
const RELATIVE_THRESHOLD_RATIO: float = 0.5

const HISTORY_BUFFER_MAXIMUM_SIZE: int = 45

const MDN_MIN_AUDIO_DECIBEL: float = -100.0  #match WebAudio defaults
const MDN_MAX_AUDIO_DECIBEL: float = -30.0  #match WebAudio defaults
const MDN_SMOOTHING: float = 0.8

var spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance
var previous_smoothed_level: float = 0.0
var level_history_buffer: PackedFloat32Array = PackedFloat32Array()
var history_levels_sum: float = 0.0
var total_frame_counter: int = 0
var last_beat_frame_counter: int = 0
var beat_event_counter: int = 0
var last_beat_time_seconds: float = 0.0

var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.MUSIC
#var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.INPUT


func _ready() -> void:
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MASTER)
    #var bus_index: int = AudioBus.get_bus_index(TARGET_AUDIO_BUS)
    var spectrum_analyzer_effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    spectrum_analyzer_effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    #spectrum_analyzer_effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_1024
    AudioEffectManager.add_effect(bus_index, spectrum_analyzer_effect)
    # EFFECTS ARE ALWAYS ADDED ON THE END OF EXISTING EFFECT ORDER!!!
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1  # index by 0
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)
    print(
        "[BeatManager] using frequency_range=",
        FREQUENCY_LOWER_BOUND_HERTZ,
        "-",
        FREQUENCY_UPPER_BOUND_HERTZ,
        "Hz"
    )


func _process(frame_delta_time: float) -> void:
    total_frame_counter += 1
    var smoothed_level: float = compute_smoothed_level_for_frequency_range(
        FREQUENCY_LOWER_BOUND_HERTZ, FREQUENCY_UPPER_BOUND_HERTZ, previous_smoothed_level
    )
    var level_rise_amount: float = smoothed_level - previous_smoothed_level
    level_history_buffer.append(smoothed_level)
    history_levels_sum += smoothed_level

    if level_history_buffer.size() > HISTORY_BUFFER_MAXIMUM_SIZE:
        history_levels_sum -= level_history_buffer[0]
        level_history_buffer.remove_at(0)

    var average_history_level: float = history_levels_sum / level_history_buffer.size()
    var relative_rise_amount: float = smoothed_level - average_history_level
    var relative_rise_ratio: float = 0.0
    if average_history_level > 0.0:
        relative_rise_ratio = relative_rise_amount / average_history_level

    if smoothed_level > ABSOLUTE_FLOOR_LEVEL and relative_rise_ratio > RELATIVE_THRESHOLD_RATIO:
        print(
            "[BeatManager] ➤ BEAT! smoothed_level=",
            smoothed_level,
            " relative_rise_ratio=",
            relative_rise_ratio,
            " frames_since_last_beat=",
            total_frame_counter - last_beat_frame_counter
        )
        _register_beat_event()

    previous_smoothed_level = smoothed_level


func _register_beat_event() -> void:
    var current_playback_time_seconds: float = _get_current_playback_time_in_seconds()
    var time_since_last_beat_in_seconds: float = 0.0
    if last_beat_time_seconds > 0.0:
        time_since_last_beat_in_seconds = current_playback_time_seconds - last_beat_time_seconds

    var bpm: float = 0.0
    if time_since_last_beat_in_seconds > 0.0:
        bpm = 60.0 / time_since_last_beat_in_seconds

    beat_detected.emit(beat_event_counter, time_since_last_beat_in_seconds, bpm)
    beat_event_counter += 1
    last_beat_time_seconds = current_playback_time_seconds
    last_beat_frame_counter = total_frame_counter


static func _get_current_playback_time_in_seconds() -> float:
    for music_player: AudioStreamPlayer in AudioPoolManager.music_pool.players:
        if music_player.playing:
            var playback_position_seconds: float = music_player.get_playback_position()
            var mix_time_seconds: float = AudioServer.get_time_since_last_mix()
            var latency_seconds: float = AudioServer.get_output_latency()
            var estimated_playback_time_seconds: float = (
                playback_position_seconds + mix_time_seconds - latency_seconds
            )
            var corrected_playback_time_seconds: float = estimated_playback_time_seconds
            if estimated_playback_time_seconds < 0.0:
                corrected_playback_time_seconds = 0.0

            return corrected_playback_time_seconds

    return 0.0


#AUXILIARIES!!!
func compute_smoothed_level_for_frequency_range(
    from_hz: float, to_hz: float, previous_smoothed_level: float
) -> float:
    var linear_avg: float = _compute_linear_average_for_frequency_range(from_hz, to_hz)
    var normalized: float = _compute_normalized_level_from_linear_magnitude(linear_avg)
    return _compute_smoothed_level(previous_smoothed_level, normalized)


func _compute_linear_average_for_frequency_range(from_hz: float, to_hz: float) -> float:
    var stereo_magnitude: Vector2 = spectrum_analyzer_instance.get_magnitude_for_frequency_range(
        from_hz, to_hz, AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
    )
    # down-mix to mono
    return (stereo_magnitude.x + stereo_magnitude.y) * 0.5


static func _compute_normalized_level_from_linear_magnitude(linear_magnitude: float) -> float:
    var db: float = linear_to_db(linear_magnitude)
    return clamp(
        (db - MDN_MIN_AUDIO_DECIBEL) / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL), 0.0, 1.0
    )


static func _compute_smoothed_level(
    previous_smoothed_level: float, new_normalized_level: float
) -> float:
    return MDN_SMOOTHING * previous_smoothed_level + (1.0 - MDN_SMOOTHING) * new_normalized_level
