extends Node
class_name IOITexture

#https://en.wikipedia.org/wiki/Time_point#Interonset_interval

#TODO: this has theoretically demonstrated the limitationss of godot engine's capabilities in
# real time audio analysis/DSP, and the issue i believe is related to noise/error compounding from
# conflicting non-deterministic relationship between the audio sampling and gdscript thread/ FRAME RATE LOOP

#it was a good attempt i believe with this but this is not going to work ever i dont think


const TEXTURE_HEIGHT: int = 2
const DEAD_CHANNEL: float = 0.0

var audio_image: Image
var audio_texture: ImageTexture

var ioi_derived_bpm_histogram: PackedInt32Array = PackedInt32Array()
const IOI_SET_SAMPLE_SIZE: float = 4.0  # how many seconds per sample group of ioi's
const MIN_BPM: float = 90.0
const MAX_BPM: float = 180.0
const BPM_BIN_COUNT: int = 16

const MIN_IOI: float = 0.30  # ≈ 200 BPM upper limit
const MAX_IOI: float = 2.0  # ≈ 30 BPM lower limit
const TEMPORAL_SMOOTHING_COEFFICIENT: float = 0.25
const MIN_ONSETS_FOR_LOCK: int = 8
const LOCK_TOLERANCE_BPM: float = 0.1
const LOCK_CONSECUTIVE: int = 4
var last_mode_bpm: float = 0.0
var consecutive_lock_count: int = 0
var bpm_locked: bool = false

var onsets: PackedFloat32Array = PackedFloat32Array()
var bpm: float = 0.0


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
        var bin_index: int = int(round((bpm_from_ioi - MIN_BPM) * bin_scale))
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
