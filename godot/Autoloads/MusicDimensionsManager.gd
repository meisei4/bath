extends Node
#class_name MusicDimensionsManager

signal onset_event(
    onset_index: int,
    time_since_previous_onset: float,
    onsets_per_minute: float,
    current_playback_time_seconds: float
)

signal tempo_event(
    beat_index_within_bar: int,
    beat_phase_within_current_beat: float,
    beats_per_minute_true_tempo: float,
    seconds_per_beat_true_tempo: float,
    seconds_per_bar_true_tempo: float
)
signal update_true_tempo(ioi_vote_counts: PackedInt32Array, most_voted_bin_index: int)

const PERCUSSIVE_FREQUENCY_LOWER_BOUND_HZ: float = 20.0
const PERCUSSIVE_FREQUENCY_UPPER_BOUND_HZ: float = 150.0
const FLUX_HISTORY_BUFFER_MAXIMUM_SIZE: int = 43  # ≈ 0.7 s @60 fps
const FLUX_THRESHOLD_RATIO: float = 1.5  # detect peaks >1.5× local average
const MIN_ONSET_INTERVAL: float = 0.25  # ignore faster repeats
const ONSETS_PER_MINUTE_HISTORY_BUFFER_MAXIMUM_SIZE: int = 8  # average over previous 8 intervals
const SECONDS_PER_MINUTE: float = 60.0

var SAMPLE_RATE: float = AudioServer.get_mix_rate()  # query real mix rate
const MDN_MIN_AUDIO_DECIBEL: float = -100.0  #match WebAudio defaults
const MDN_MAX_AUDIO_DECIBEL: float = -30.0  #match WebAudio defaults
const MDN_SMOOTHING: float = 0.8

var spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance
var previous_smoothed_level: float = 0.0
var flux_history_buffer: PackedFloat32Array = PackedFloat32Array()
var onset_intervals_history_buffer: PackedFloat32Array = PackedFloat32Array()
var time_of_previous_onset: float = 0.0
var onset_event_counter: int = 0


func _ready() -> void:
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)


func _process(delta_time: float) -> void:
    var spectral_flux: float = compute_spectral_flux()
    update_flux_history(spectral_flux)
    if (
        spectral_flux > current_flux_threshold()
        and time_since_previous_onset() > MIN_ONSET_INTERVAL
    ):
        _emit_rhythm_signals()


func compute_spectral_flux() -> float:
    var smoothed_level: float = compute_smoothed_level_for_frequency_range(
        PERCUSSIVE_FREQUENCY_LOWER_BOUND_HZ,
        PERCUSSIVE_FREQUENCY_UPPER_BOUND_HZ,
        previous_smoothed_level
    )
    var flux: float = smoothed_level - previous_smoothed_level
    if flux < 0.0:
        flux = 0.0

    previous_smoothed_level = smoothed_level
    return flux


func update_flux_history(flux: float) -> void:
    flux_history_buffer.append(flux)
    if flux_history_buffer.size() > FLUX_HISTORY_BUFFER_MAXIMUM_SIZE:
        flux_history_buffer.remove_at(0)


func current_flux_threshold() -> float:
    var sum: float = 0.0
    for flux: float in flux_history_buffer:
        sum += flux
    var average_flux: float = 0.0
    if flux_history_buffer.size() > 0:
        average_flux = sum / flux_history_buffer.size()

    return average_flux * FLUX_THRESHOLD_RATIO


func time_since_previous_onset() -> float:
    var current_playback_time: float = get_current_playback_time()
    var time_since_previous_onset: float = 1e6  #???? 0.0
    if time_of_previous_onset > 0.0:
        time_since_previous_onset = current_playback_time - time_of_previous_onset
    return time_since_previous_onset


func _emit_rhythm_signals() -> void:
    var current_playback_time: float = get_current_playback_time()
    var time_since_previous_onset: float = time_since_previous_onset()
    onset_intervals_history_buffer.append(time_since_previous_onset)
    if onset_intervals_history_buffer.size() > ONSETS_PER_MINUTE_HISTORY_BUFFER_MAXIMUM_SIZE:
        onset_intervals_history_buffer.remove_at(0)

    var sum: float = 0.0
    for delta_time: float in onset_intervals_history_buffer:
        sum += delta_time

    var average_interval: float = 0.0
    if onset_intervals_history_buffer.size() > 0:
        average_interval = sum / onset_intervals_history_buffer.size()

    var onsets_per_minute: float = 0.0
    if average_interval > 0.0:
        onsets_per_minute = SECONDS_PER_MINUTE / average_interval

    onset_event.emit(
        onset_event_counter, time_since_previous_onset, onsets_per_minute, current_playback_time
    )
    onset_event_counter += 1
    time_of_previous_onset = current_playback_time


static func get_current_playback_time() -> float:
    for player: AudioStreamPlayer in AudioPoolManager.music_pool.players:
        if player.playing:
            var current_playback_position: float = player.get_playback_position()
            var time_since_previous_mix: float = AudioServer.get_time_since_last_mix()
            var output_latency: float = AudioServer.get_output_latency()
            var estimated_playback_time: float = (
                current_playback_position + time_since_previous_mix - output_latency
            )
            var current_playback_time: float = 0.0
            if estimated_playback_time > 0.0:
                current_playback_time = estimated_playback_time
            return current_playback_time
    return 0.0


#AUXILIARIES!!!
func compute_smoothed_level_for_frequency_range(
    from_hz: float, to_hz: float, previous_smoothed_level: float
) -> float:
    var linear_average: float = _compute_linear_average_for_frequency_range(from_hz, to_hz)
    var normalized: float = _compute_normalized_level_from_linear_magnitude(linear_average)
    return _compute_smoothed_level(previous_smoothed_level, normalized)


func _compute_linear_average_for_frequency_range(from_hz: float, to_hz: float) -> float:
    var stereo_magnitude: Vector2 = spectrum_analyzer_instance.get_magnitude_for_frequency_range(
        from_hz, to_hz, AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
    )
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


## DECOMPOSITION AUXILARIES
# TODO: these can change over time depending on the song structure... how do we account for that??
var TIME_SIGNATURE_NUMERATOR: int = 4  # default “4/4”
var TIME_SIGNATURE_DENOMINATOR: int = 4
var SUBDIVISIONS_PER_onset: int = 1  # default = sixteenth-notes


func _decompose_percussive_onsets(onsets_per_minute: float) -> void:
    var time_signature: Vector2i = Vector2i(TIME_SIGNATURE_NUMERATOR, TIME_SIGNATURE_DENOMINATOR)
    var seconds_per_onset: float = SECONDS_PER_MINUTE / onsets_per_minute
    var seconds_per_subdivision: float = seconds_per_onset / SUBDIVISIONS_PER_onset
    var seconds_per_bar: float = seconds_per_onset * TIME_SIGNATURE_NUMERATOR
