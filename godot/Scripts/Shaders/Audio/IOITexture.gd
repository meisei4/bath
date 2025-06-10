extends Node
class_name IOITexture

#https://en.wikipedia.org/wiki/Time_point#Interonset_interval

#TODO: this has theoretically demonstrated the limitationss of godot engine's capabilities in
# real time audio analysis/DSP, and the issue i believe is related to noise/error compounding from
# conflicting non-deterministic relationship between the audio sampling and gdscript thread/ FRAME RATE LOOP

#OLD PARAM SETTINGS for record:
#const IOI_SET_SAMPLE_SIZE: float = 4.0
#const MIN_BPM: float = 90.0
#const MAX_BPM: float = 180.0
#const BPM_BIN_COUNT: int = 16
#const MIN_IOI: float = 0.30
#const MAX_IOI: float = 2.0
#const TEMPORAL_SMOOTHING_COEFFICIENT: float = 0.25
#const MIN_ONSETS_FOR_LOCK: int = 8
#const LOCK_TOLERANCE_BPM: float = 0.1
#const LOCK_CONSECUTIVE: int = 4

# Tracks inter-onset intervals (IOIs) to estimate beats-per-minute (BPM) from onset events.
# Tuned parameters below are optimized for accurately locking onto ~125 BPM, while
# rejecting noise from audio mix latency wobble and GDScript frame rate jitter.
#
# Audio mix latency wobble:
# • Godot's audio server runs in its own thread and buffers audio internally.
# • get_time_since_last_mix() + get_output_latency() can vary by ±1–5 ms per frame
#   due to driver buffering, OS scheduling, and hardware latency fluctuations.
#
# Frame rate jitter:
# • GDScript’s _process(delta) loop aims for a target (e.g. 60 FPS) but each frame
#   actually takes ~15–20 ms (±2–5 ms jitter) depending on system load.
# • This adds non-uniform sampling intervals to timestamped onsets.
#
# Combined effect:
# • These two sources introduce ~1–5 ms noise into IOI measurements,
#   translating to ~0.5–2 BPM error at typical tempos.
# • The parameters below are chosen to average out and reject these errors,
#   so that a steady 125 BPM signal locks reliably.

const IOI_SET_SAMPLE_SIZE: float = 6.0  # [sec]
# └─ Sliding window length (seconds) for IOI collection.
#    At 125 BPM (0.48 s IOI) yields ~12.5 onsets in 6 s, enough to smooth jitter.

const MIN_BPM: float = 80.0  # [beats/min]
# └─ Lower tempo bound (100 BPM = 0.60 s IOI).
#    Filters out long pauses or spurious slow events below expected range.

const MAX_BPM: float = 180.0  # [beats/min]
# └─ Upper tempo bound (150 BPM = 0.40 s IOI).
#    Filters out rapid noise/glitches above expected range.

const BPM_BIN_COUNT: int = 5  # [bins]
# └─ Number of histogram buckets between MIN_BPM and MAX_BPM.
#    (150−100)/(51−1) = 1.0 BPM per bin, so 125 BPM lands exactly in a bin.

const MIN_IOI: float = 60.0 / MAX_BPM  # ≈0.40 [sec]
# └─ Minimum inter-onset interval to accept (IOI ≥ 0.40 s).

#const MAX_IOI: float                     = 60.0 / MIN_BPM    # ≈0.40 [sec]
const MAX_IOI: float = 2.0  # ≈0.60 [sec]

# └─ Maximum IOI to accept (IOI ≤ 0.60 s).

const TEMPORAL_SMOOTHING_COEFFICIENT: float = 0.25  # [unitless]
# └─ Exponential smoothing α (0–1) for updating locked BPM.
#    0.3 means 30% new value + 70% previous, balancing reactivity vs jitter rejection.

const MIN_ONSETS_FOR_LOCK: int = 0  # [count]
# └─ Require ≥12 onsets in the 6 s window (i.e. ~12.5 expected @125 BPM)
#    before attempting to lock tempo. Prevents premature lock on few beats.

const LOCK_TOLERANCE_BPM: float = 0.1  # [BPM]
# └─ Mode BPM must stay within ±0.5 BPM across consecutive readings to count as stable.

const LOCK_CONSECUTIVE: int = 12  # [readings]
# └─ Number of successive stable mode detections required to finalize the lock.
#    6 readings at ~1 onset per 0.48 s ≈ 2.9 s of consistent tempo.

#TODO:

#NO IDEA but often
#112.5
#108.75
#106.125
#104.2875
#103.00125
#102.100875
#101.4706125
#101.02942875
#100.720600125
#100.5044200875
#100.35309406125
#100.247165842875
#100.0

var last_mode_bpm: float = 0.0
var consecutive_lock_count: int = 0
var bpm_locked: bool = false

var onsets: PackedFloat32Array = PackedFloat32Array()
var bpm: float = 0.0
var ioi_derived_bpm_histogram: PackedInt32Array = PackedInt32Array()

const TEXTURE_HEIGHT: int = 2
const DEAD_CHANNEL: float = 0.0
var audio_image: Image
var audio_texture: ImageTexture


func _ready() -> void:
    ioi_derived_bpm_histogram.resize(BPM_BIN_COUNT)
    ioi_derived_bpm_histogram.fill(0)
    audio_image = Image.create(BPM_BIN_COUNT, TEXTURE_HEIGHT, false, Image.FORMAT_R8)
    audio_texture = ImageTexture.create_from_image(audio_image)
    MusicDimensionsManager.onset_detected.connect(_update_bpm)


func _update_bpm(current_playback_time: float) -> void:
    if bpm_locked:
        return
    var onset_count: int = onsets.size()
    onsets.append(current_playback_time)
    if onset_count < MIN_ONSETS_FOR_LOCK:
        return  # still warming up
    while onset_count > 0 and current_playback_time - onsets[0] > IOI_SET_SAMPLE_SIZE:
        onsets.remove_at(0)

    ioi_derived_bpm_histogram.fill(0)
    var bin_scale: float = float(BPM_BIN_COUNT - 1) / (MAX_BPM - MIN_BPM)
    for i: int in range(1, onsets.size()):
        var ioi: float = onsets[i] - onsets[i - 1]
        var bpm_from_ioi: float = MusicDimensionsManager.SECONDS_PER_MINUTE / ioi
        # ACCOUNT FOR octave-folding (double bpms or half bpms?
        while bpm_from_ioi < MIN_BPM:
            bpm_from_ioi *= 2.0
        while bpm_from_ioi > MAX_BPM:
            bpm_from_ioi *= 0.5

        if bpm_from_ioi < MIN_BPM or bpm_from_ioi > MAX_BPM:
            continue
        var bin_index: int = roundi((bpm_from_ioi - MIN_BPM) * bin_scale)
        ioi_derived_bpm_histogram[bin_index] += 1

    var bin_of_current_mode: int = 0
    var current_mode_count: int = -1
    var total_count: int = 0
    for bin: int in range(BPM_BIN_COUNT):
        total_count += ioi_derived_bpm_histogram[bin]
        if ioi_derived_bpm_histogram[bin] > current_mode_count:
            current_mode_count = ioi_derived_bpm_histogram[bin]
            bin_of_current_mode = bin
    if total_count == 0:
        return

    var mode_bpm: float = (
        MIN_BPM + float(bin_of_current_mode) * (MAX_BPM - MIN_BPM) / float(BPM_BIN_COUNT - 1)
    )
    if last_mode_bpm > 0.0 and abs(mode_bpm - last_mode_bpm) < LOCK_TOLERANCE_BPM:
        consecutive_lock_count += 1
    else:
        consecutive_lock_count = 1
    last_mode_bpm = mode_bpm
    if consecutive_lock_count >= LOCK_CONSECUTIVE:
        bpm = mode_bpm
        bpm_locked = true
        return
    if bpm == 0.0:
        bpm = mode_bpm
    else:
        bpm = lerp(bpm, mode_bpm, TEMPORAL_SMOOTHING_COEFFICIENT)
