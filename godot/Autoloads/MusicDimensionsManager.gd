extends Node
#class_name MusicDimensionsManager

signal rhythm_indicator(beat_index: int, bar_index: int, beats_per_minute: float)

var time_signature: int = 8
var onset_detection_threshold: float = 0.1  # Minimum rise in audio level to count as a beat
var audio_capture_interval_seconds: float = 0.0333333  # Seconds per capture (~30 Hz)

const CAPTURE_SAMPLE_COUNT: int = 512
const INTERVAL_HISTORY_MAX: int = 8

var _audio_effect_capture: AudioEffectCapture = null
var _previous_audio_level: float = 0.0
var _last_onset_time_milliseconds: float = 0.0
var _interval_history_milliseconds: Array[float] = []
var _detected_beat_index: int = 0

var BPM: float = 120.0


func _ready() -> void:
    _initialize_audio_capture()


func _initialize_audio_capture() -> void:
    _audio_effect_capture = AudioEffectCapture.new()
    _audio_effect_capture.buffer_length = audio_capture_interval_seconds
    var music_bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    AudioServer.add_bus_effect(music_bus_index, _audio_effect_capture)


func _process(delta_time: float) -> void:
    if not _audio_effect_capture.can_get_buffer(CAPTURE_SAMPLE_COUNT):
        return

    var captured_frames: PackedVector2Array = _audio_effect_capture.get_buffer(CAPTURE_SAMPLE_COUNT)
    var sum_absolute_levels: float = 0.0
    for frame: Vector2 in captured_frames:
        sum_absolute_levels += abs(frame.x)  # assume mono in .x

    var current_audio_level: float = sum_absolute_levels / float(CAPTURE_SAMPLE_COUNT)
    var audio_level_rise: float = current_audio_level - _previous_audio_level

    if audio_level_rise > onset_detection_threshold:
        _handle_beat_detection()

    _previous_audio_level = current_audio_level


func _handle_beat_detection() -> void:
    var precise_time_seconds: float = _get_precise_audio_playback_time()
    var precise_time_milliseconds: float = precise_time_seconds * 1000.0

    if _last_onset_time_milliseconds > 0.0:
        var interval_milliseconds: float = precise_time_milliseconds - _last_onset_time_milliseconds
        _interval_history_milliseconds.append(interval_milliseconds)
        if _interval_history_milliseconds.size() > INTERVAL_HISTORY_MAX:
            _interval_history_milliseconds.pop_front()

        var total_interval_milliseconds: float = 0.0
        for interval: float in _interval_history_milliseconds:
            total_interval_milliseconds += interval

        var average_interval_milliseconds: float = (
            total_interval_milliseconds / float(_interval_history_milliseconds.size())
        )
        BPM = 60000.0 / average_interval_milliseconds

    _last_onset_time_milliseconds = precise_time_milliseconds

    var current_bar_index: int = _detected_beat_index / time_signature
    rhythm_indicator.emit(_detected_beat_index, current_bar_index, BPM)
    #print("▶️ Beat #", _detected_beat_index, " Bar #", current_bar_index, "  BPM=", BPM)
    _detected_beat_index += 1


static func _get_precise_audio_playback_time() -> float:
    for music_player: AudioStreamPlayer in AudioManager.music_pool:
        if music_player.playing:
            var playback_position_seconds: float = music_player.get_playback_position()
            var time_since_last_mix_seconds: float = AudioServer.get_time_since_last_mix()
            var output_latency_seconds: float = AudioServer.get_output_latency()

            playback_position_seconds += time_since_last_mix_seconds
            playback_position_seconds -= output_latency_seconds

            return max(playback_position_seconds, 0.0)

    return 0.0
