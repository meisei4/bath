extends Node
#class_name MusicDimensionsManager

signal beat_detected(beat_index: int, time_since_last_beat_in_seconds: float, bpm: float)
#TODO: merge these signals???
signal tempo_detected(
    time_signature: Vector2i,
    subdivisions_per_beat: int,
    bpm: float,
    seconds_per_beat: float,
    seconds_per_subdivision: float,
    seconds_per_bar: float
)

const FREQUENCY_LOWER_BOUND_HERTZ: float       = 20.0
const FREQUENCY_UPPER_BOUND_HERTZ: float       = 150.0
const FLUX_HISTORY_BUFFER_MAXIMUM_SIZE: int    = 43       # ≈ 0.7 s @60 fps
const FLUX_THRESHOLD_RATIO: float              = 1.5      # detect peaks >1.5× local mean
const MIN_BEAT_INTERVAL_SECONDS: float         = 0.25     # ignore faster repeats
const BPM_HISTORY_BUFFER_MAXIMUM_SIZE: int     = 8        # average over last 8 intervals
const SECONDS_PER_MINUTE: float = 60.0

#TODO: cant these change over time depending on the song structure???? how do we account for that??
var TIME_SIGNATURE_NUMERATOR:   int   = 4   # default “4/4”
var TIME_SIGNATURE_DENOMINATOR: int   = 4
var SUBDIVISIONS_PER_BEAT:      int   = 1   # default = sixteenth-notes

var SAMPLE_RATE: float = AudioServer.get_mix_rate()  # query real mix rate
const MDN_MIN_AUDIO_DECIBEL: float = -100.0  #match WebAudio defaults
const MDN_MAX_AUDIO_DECIBEL: float = -30.0  #match WebAudio defaults
const MDN_SMOOTHING: float = 0.8


var spectrum_analyzer_instance: AudioEffectSpectrumAnalyzerInstance
var previous_smoothed_level: float                   = 0.0
var flux_history_buffer: PackedFloat32Array     = PackedFloat32Array()
var interval_history_buffer: PackedFloat32Array = PackedFloat32Array()
var last_beat_time_seconds: float               = 0.0
var beat_event_counter: int                     = 0

func _ready() -> void:
    # Attach a spectrum analyzer to the MUSIC bus
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)
    print("[MusicDimensionsManager] Spectral-flux detector active in",
          FREQUENCY_LOWER_BOUND_HERTZ, "–", FREQUENCY_UPPER_BOUND_HERTZ, "Hz")

func _process(delta_time: float) -> void:
    var flux: float = compute_spectral_flux()
    update_flux_history(flux)
    if flux > current_flux_threshold() and time_since_last_beat() > MIN_BEAT_INTERVAL_SECONDS:
        _emit_beat_signal()

func compute_spectral_flux() -> float:
    var smoothed_level: float = compute_smoothed_level_for_frequency_range(
        FREQUENCY_LOWER_BOUND_HERTZ,
        FREQUENCY_UPPER_BOUND_HERTZ,
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
    for f in flux_history_buffer:
        sum += f
    var mean: float = 0.0
    if(flux_history_buffer.size() > 0):
        mean =  sum / flux_history_buffer.size()

    return mean * FLUX_THRESHOLD_RATIO


func time_since_last_beat() -> float:
    var current_playback_time_in_seconds: float = _get_current_playback_time_in_seconds()
    var time_since_last_beat: float = 1e6 #????
    if(last_beat_time_seconds > 0.0):
        time_since_last_beat = current_playback_time_in_seconds - last_beat_time_seconds
    return time_since_last_beat

func _emit_beat_signal() -> void:
    var current_playback_time_in_seconds: float = _get_current_playback_time_in_seconds()
    var interval: float = 0.0
    if(last_beat_time_seconds > 0.0):
        interval = current_playback_time_in_seconds - last_beat_time_seconds

    interval_history_buffer.append(interval)
    if interval_history_buffer.size() > BPM_HISTORY_BUFFER_MAXIMUM_SIZE:
        interval_history_buffer.remove_at(0)
    var sum: float = 0.0
    for dt in interval_history_buffer:
        sum += dt
    var avg_interval: float = 0.0
    if(interval_history_buffer.size() > 0):
        avg_interval = sum / interval_history_buffer.size()
    var bpm: float = 0.0
    if(avg_interval > 0.0):
        bpm = SECONDS_PER_MINUTE / avg_interval

    beat_detected.emit(beat_event_counter, interval, bpm)
    _emit_tempo_signal(bpm)
    beat_event_counter += 1
    last_beat_time_seconds = current_playback_time_in_seconds

static func _get_current_playback_time_in_seconds() -> float:
    for player: AudioStreamPlayer in AudioPoolManager.music_pool.players:
        if player.playing:
            var current_playback_position: float = player.get_playback_position()
            var time_since_last_mix: float = AudioServer.get_time_since_last_mix()
            var output_latency: float = AudioServer.get_output_latency()
            var estimated_playback_time_in_seconds: float = current_playback_position + time_since_last_mix - output_latency
            var current_playback_time_in_seconds: float = 0.0
            if(estimated_playback_time_in_seconds > 0.0):
                current_playback_time_in_seconds = estimated_playback_time_in_seconds
            return current_playback_time_in_seconds
    return 0.0


func _emit_tempo_signal(bpm: float) -> void:
    var seconds_per_beat: float = SECONDS_PER_MINUTE / bpm
    var seconds_per_subdivision: float = seconds_per_beat / SUBDIVISIONS_PER_BEAT
    var seconds_per_bar: float = seconds_per_beat * TIME_SIGNATURE_NUMERATOR

    tempo_detected.emit(
        Vector2i(TIME_SIGNATURE_NUMERATOR, TIME_SIGNATURE_DENOMINATOR),
        SUBDIVISIONS_PER_BEAT,
        bpm,
        seconds_per_beat,
        seconds_per_subdivision,
        seconds_per_bar
    )


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
