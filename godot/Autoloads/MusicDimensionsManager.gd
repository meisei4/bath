extends Node
#class_name MusicDimensionsManager

signal onset_detected(current_playback_time: float)

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
var previous_smooth_energy: float = 0.0
var flux_novelty_curve: PackedFloat32Array = PackedFloat32Array()
var onset_intervals_history_buffer: PackedFloat32Array = PackedFloat32Array()
var time_of_previous_onset: float = 0.0
var onset_event_counter: int = 0

var metronome_click: AudioStream = preload(AudioConsts.METRONOME_CLICK)
var time_of_next_click: float = 0.0
var rust_util: RustUtil
var bpm: float
var song_time: float = 0.0

#var song: String = AudioConsts.SHADERTOY_MUSIC_TRACK_EXPERIMENT_WAV
#var song: String = AudioConsts.HELLION_WAV
var song: String = AudioConsts.SNUFFY


func _ready() -> void:
    rust_util = RustUtil.new()
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)

    derive_bpm()
    #isolate_melody()
    load_custom_onsets()
    var music_resource: AudioStream = load(song)
    AudioPoolManager.play_music(music_resource)
    #var input_resource: AudioStreamMicrophone = AudioStreamMicrophone.new()
    #AudioManager.play_input(input_resource, 0.0)
    #TODO: ^^^ ew, figure out how to perhaps make it more obvious that the audio texture can target whatever audio bus...


func derive_bpm() -> void:
    var wav_fs_path = ProjectSettings.globalize_path(song)
    bpm = rust_util.detect_bpm(wav_fs_path)
    print("aubio derived bpm is:", bpm)

var ioi: float = 60.0 / bpm
func debug_bpm_onsets(delta: float) -> void:
    time_of_next_click -= delta
    if time_of_next_click <= 0.0:
        AudioPoolManager.play_sfx(metronome_click)
        time_of_next_click += ioi


#TODO: hacked non-working melody isolator, need to use spleeter machine learning stuff...
var melody_onsets: PackedFloat32Array = []
var melody_index: int = 0


func isolate_melody() -> void:
    var wav_fs_path = ProjectSettings.globalize_path(song)
    melody_onsets = rust_util.isolate_melody(wav_fs_path, 1200.0)
    #print("melody onsets are: ", melody_onsets)
    melody_index = 0
    song_time = 0.0


func debug_melody_onsets(delta: float) -> void:
    if melody_index >= melody_onsets.size():
        return

    song_time += delta
    if (song_time >= melody_onsets[melody_index]):
        AudioPoolManager.play_sfx(metronome_click)
        melody_index += 1

static var custom_onsets_flat_buffer: PackedVector4Array

static func load_custom_onsets() -> void:
    var res: RhythmOnsetData = ResourceLoader.load("res://Resources/Audio/CustomOnsets/custom_onsets.tres")
    custom_onsets_flat_buffer.clear()
    var uki      = res.uki
    var shizumi  = res.shizumi
    var total_onsets = min(uki.size(), shizumi.size()) / 2
    for i: int in range(int(total_onsets)):
        var u_start = uki[i*2]
        var u_end   = uki[i*2+1]
        var s_start = shizumi[i*2]
        var s_end   = shizumi[i*2+1]

        var uki_flat = Vector2(u_start, u_end)
        var shizumi_flat  = Vector2(s_start, s_end)

        var uki_shizumi_flat = Vector4(uki_flat.x,     uki_flat.y,
                                       shizumi_flat.x, shizumi_flat.y)
        custom_onsets_flat_buffer.append(uki_shizumi_flat)


static var uki_onset_index: int = 0
static var shizumi_onset_index: int = 0
func debug_custom_onsets(delta: float) -> void:
    song_time += delta

    while uki_onset_index < custom_onsets_flat_buffer.size():
        var next_uki_onset = custom_onsets_flat_buffer[uki_onset_index].x
        if song_time < next_uki_onset:
            break
        AudioPoolManager.play_sfx(metronome_click)
        uki_onset_index += 1

    while shizumi_onset_index < custom_onsets_flat_buffer.size():
        var next_j_start = custom_onsets_flat_buffer[shizumi_onset_index].z
        if song_time < next_j_start:
            break
        AudioPoolManager.play_sfx(metronome_click)
        shizumi_onset_index += 1

#TODO: this identical to ManualRhythmOnsetRecorder._debug_keys()
func debug_custom_onsets_ASCII(delta: float) -> void:
    var prev_time = song_time
    song_time += delta
    var f_char      : String = " "
    var j_char      : String = " "
    var f_press_fmt : String = ""
    var f_rel_fmt   : String = ""
    var j_press_fmt : String = ""
    var j_rel_fmt   : String = ""
    for v in custom_onsets_flat_buffer:
        var u_start = v.x
        var u_end   = v.y
        var s_start = v.z
        var s_end   = v.w
        if prev_time < u_start and song_time >= u_start:
            f_char      = "F"
            f_press_fmt = "F_PRS:[%.3f,      ]" % u_start
        if prev_time < u_end and song_time >= u_end:
            f_rel_fmt   = "F_REL:[%.3f, %.3f]" % [u_start, u_end]
        if prev_time < s_start and song_time >= s_start:
            j_char      = "J"
            j_press_fmt = "J_PRS:[%.3f,      ]" % s_start

        if prev_time < s_end and song_time >= s_end:
            j_rel_fmt   = "J_REL:[%.3f, %.3f]" % [s_start, s_end]
    var event_body  = f_press_fmt + f_rel_fmt + j_press_fmt + j_rel_fmt
    var status_body = "[%s] [%s]" % [f_char, j_char]
    if event_body != "":
        status_body += "   " + event_body
        print(status_body)



func _process(delta_time: float) -> void:
    #TODO: LMAO these are expensive and should not be called every frame:
    #https://docs.godotengine.org/en/stable/classes/class_audioserver.html#class-audioserver-method-get-output-latency
    var time_since_previous_mix: float = AudioServer.get_time_since_last_mix()
    var output_latency: float = AudioServer.get_output_latency()
    var current_playback_time: float = get_current_playback_time(
        time_since_previous_mix, output_latency
    )

    var flux: float = compute_flux()
    update_flux_novelty_curve(flux)
    if (
        flux > current_flux_threshold()
        and time_since_previous_onset(current_playback_time) > MIN_ONSET_INTERVAL
    ):
        emit_rhythm_signals(current_playback_time)


func compute_flux() -> float:
    var smooth_energy: float = compute_smooth_energy_for_frequency_range(
        PERCUSSIVE_FREQUENCY_LOWER_BOUND_HZ,
        PERCUSSIVE_FREQUENCY_UPPER_BOUND_HZ,
        previous_smooth_energy
    )
    var flux: float = smooth_energy - previous_smooth_energy
    if flux < 0.0:
        flux = 0.0

    previous_smooth_energy = smooth_energy
    return flux


#this is a ring buffer!!!
func update_flux_novelty_curve(flux: float) -> void:
    flux_novelty_curve.append(flux)
    if flux_novelty_curve.size() > FLUX_HISTORY_BUFFER_MAXIMUM_SIZE:
        flux_novelty_curve.remove_at(0)


func current_flux_threshold() -> float:
    var sum: float = 0.0
    for flux: float in flux_novelty_curve:
        sum += flux
    var average_flux: float = 0.0
    if flux_novelty_curve.size() > 0:
        average_flux = sum / flux_novelty_curve.size()

    return average_flux * FLUX_THRESHOLD_RATIO


func emit_rhythm_signals(current_playback_time: float) -> void:
    var time_since_previous_onset: float = time_since_previous_onset(current_playback_time)
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

    onset_detected.emit(current_playback_time)
    onset_event_counter += 1
    time_of_previous_onset = current_playback_time


func time_since_previous_onset(current_playback_time: float) -> float:
    var time_since_previous_onset: float = 1e6  #???? 0.0
    if self.time_of_previous_onset > 0.0:
        time_since_previous_onset = current_playback_time - self.time_of_previous_onset
    return time_since_previous_onset


static func get_current_playback_time(
    time_since_previous_mix: float, output_latency: float
) -> float:
    for player: AudioStreamPlayer in AudioPoolManager.music_pool.players:
        if player.playing:
            var current_playback_position: float = player.get_playback_position()
            var estimated_playback_time: float = (
                current_playback_position + time_since_previous_mix - output_latency
            )
            var current_playback_time: float = 0.0
            if estimated_playback_time > 0.0:
                current_playback_time = estimated_playback_time
            return current_playback_time
    return 0.0


#AUXILIARIES!!!
func compute_smooth_energy_for_frequency_range(
    from_hz: float, to_hz: float, previous_smooth_energy: float
) -> float:
    var linear_average: float = _compute_linear_average_for_frequency_range(from_hz, to_hz)
    var normalized: float = _compute_normalized_energy_from_linear_magnitude(linear_average)
    return _compute_smooth_energy(previous_smooth_energy, normalized)


func _compute_linear_average_for_frequency_range(from_hz: float, to_hz: float) -> float:
    var stereo_magnitude: Vector2 = spectrum_analyzer_instance.get_magnitude_for_frequency_range(
        from_hz, to_hz, AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
    )
    return (stereo_magnitude.x + stereo_magnitude.y) * 0.5


static func _compute_normalized_energy_from_linear_magnitude(linear_magnitude: float) -> float:
    var db: float = linear_to_db(linear_magnitude)
    return clamp(
        (db - MDN_MIN_AUDIO_DECIBEL) / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL), 0.0, 1.0
    )


static func _compute_smooth_energy(
    previous_smooth_energy: float, new_normalized_energy: float
) -> float:
    return MDN_SMOOTHING * previous_smooth_energy + (1.0 - MDN_SMOOTHING) * new_normalized_energy


## DECOMPOSITION AUXILARIES
# TODO: these can change over time depending on the song structure... how do we account for that??
var TIME_SIG_N: int = 4  # default “4/4”
var TIME_SIG_D: int = 4
var SUBDIVISIONS_PER_onset: int = 1  # default = sixteenth-notes


func _decompose_percussive_onsets(onsets_per_minute: float) -> void:
    var time_signature: Vector2i = Vector2i(TIME_SIG_N, TIME_SIG_D)
    var seconds_per_onset: float = SECONDS_PER_MINUTE / onsets_per_minute
    var seconds_per_subdivision: float = seconds_per_onset / SUBDIVISIONS_PER_onset
    var seconds_per_bar: float = seconds_per_onset * TIME_SIG_N
