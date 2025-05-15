extends Node
class_name TempoDimension
#https://staff.aist.go.jp/m.goto/PAPER/IEEEPROC200804goto.pdf

const ANALYSIS_WINDOW_DURATION_SECONDS: float = 4.0
const HISTOGRAM_BIN_DURATION_MILLISECONDS: int = 20
const MINIMUM_TEMPO_BPM: float = 60.0
const MAXIMUM_TEMPO_BPM: float = 200.0
const TEMPORAL_SMOOTHING_COEFFICIENT: float = 0.25

var percussive_onset_timestamps: PackedFloat32Array = PackedFloat32Array()
var beats_per_minute_true_tempo: float = 0.0
var absolute_time_of_last_aligned_beat: float = -1.0
var global_beat_counter: int = 0
const IOI_BIN_COUNT: int = 32
const IOI_BIN_DURATION_SECONDS: float = 0.020  # 20 ms

var ioi_vote_counts: PackedInt32Array = PackedInt32Array()


func _ready() -> void:
    ioi_vote_counts.resize(IOI_BIN_COUNT)
    ioi_vote_counts.fill(0)
    MusicDimensionsManager.onset_event.connect(_on_percussive_onset_event)


func _on_percussive_onset_event(
    onset_index: int,
    time_since_previous_onset_seconds: float,
    onsets_per_minute_instantaneous: float,
    current_playback_time_seconds: float
) -> void:
    _update_true_tempo_estimate(current_playback_time_seconds)
    _align_internal_metronome_if_needed(current_playback_time_seconds)


func _process(delta_time_seconds: float) -> void:
    if beats_per_minute_true_tempo == 0.0:
        return

    var seconds_per_beat_true_tempo: float = (
        MusicDimensionsManager.SECONDS_PER_MINUTE / beats_per_minute_true_tempo
    )
    var seconds_per_bar_true_tempo: float = (
        seconds_per_beat_true_tempo * MusicDimensionsManager.TIME_SIGNATURE_NUMERATOR
    )
    var current_time_seconds: float = MusicDimensionsManager.get_current_playback_time()
    var beat_phase_within_current_beat: float = clamp(
        (current_time_seconds - absolute_time_of_last_aligned_beat) / seconds_per_beat_true_tempo,
        0.0,
        1.0
    )

    if beat_phase_within_current_beat >= 1.0:
        beat_phase_within_current_beat -= 1.0
        absolute_time_of_last_aligned_beat += seconds_per_beat_true_tempo
        global_beat_counter += 1

    var beat_index_within_bar: int = (
        global_beat_counter % MusicDimensionsManager.TIME_SIGNATURE_NUMERATOR
    )

    MusicDimensionsManager.tempo_event.emit(
        beat_index_within_bar,
        beat_phase_within_current_beat,
        beats_per_minute_true_tempo,
        seconds_per_beat_true_tempo,
        seconds_per_bar_true_tempo
    )


func _update_true_tempo_estimate(current_time_seconds: float) -> void:
    percussive_onset_timestamps.append(current_time_seconds)
    while (
        percussive_onset_timestamps.size() > 0
        and current_time_seconds - percussive_onset_timestamps[0] > ANALYSIS_WINDOW_DURATION_SECONDS
    ):
        percussive_onset_timestamps.remove_at(0)

    ioi_vote_counts.fill(0)

    for index: int in range(percussive_onset_timestamps.size() - 1):
        var period_seconds: float = current_time_seconds - percussive_onset_timestamps[index]
        if period_seconds <= 0.0:
            continue
        var candidate_bpm: float = MusicDimensionsManager.SECONDS_PER_MINUTE / period_seconds
        if candidate_bpm < MINIMUM_TEMPO_BPM or candidate_bpm > MAXIMUM_TEMPO_BPM:
            continue
        var bin_index: int = int(
            round(period_seconds * 1000.0 / HISTOGRAM_BIN_DURATION_MILLISECONDS)
        )
        if bin_index >= 0 and bin_index < ioi_vote_counts.size():
            ioi_vote_counts[bin_index] = ioi_vote_counts[bin_index] + 1

    var most_voted_bin_index: int = -1
    var highest_vote_count: int = -1
    for bin_index_key: int in range(ioi_vote_counts.size()):
        var vote_count: int = ioi_vote_counts[bin_index_key]
        if vote_count > highest_vote_count:
            highest_vote_count = vote_count
            most_voted_bin_index = bin_index_key

    if highest_vote_count <= 0:
        return

    var dominant_period_seconds: float = (
        (most_voted_bin_index * HISTOGRAM_BIN_DURATION_MILLISECONDS) / 1000.0
    )
    var dominant_bpm_raw: float = (
        MusicDimensionsManager.SECONDS_PER_MINUTE / dominant_period_seconds
    )

    if beats_per_minute_true_tempo == 0.0:
        beats_per_minute_true_tempo = dominant_bpm_raw
    else:
        beats_per_minute_true_tempo = lerp(
            beats_per_minute_true_tempo, dominant_bpm_raw, TEMPORAL_SMOOTHING_COEFFICIENT
        )

    MusicDimensionsManager.update_true_tempo.emit(ioi_vote_counts, most_voted_bin_index)


func _align_internal_metronome_if_needed(current_time_seconds: float) -> void:
    if beats_per_minute_true_tempo == 0.0:
        return

    var seconds_per_beat_true_tempo: float = (
        MusicDimensionsManager.SECONDS_PER_MINUTE / beats_per_minute_true_tempo
    )
    if absolute_time_of_last_aligned_beat < 0.0:
        absolute_time_of_last_aligned_beat = current_time_seconds
        global_beat_counter = 0
        return

    var phase_error_seconds: float = fmod(
        current_time_seconds - absolute_time_of_last_aligned_beat, seconds_per_beat_true_tempo
    )
    if (
        phase_error_seconds < 0.25 * seconds_per_beat_true_tempo
        or phase_error_seconds > 0.75 * seconds_per_beat_true_tempo
    ):
        absolute_time_of_last_aligned_beat = current_time_seconds
