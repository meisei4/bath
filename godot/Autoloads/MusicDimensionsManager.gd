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

var custom_onset_data: RhythmOnsetData = preload(
    "res://Resources/Audio/CustomOnsets/custom_onsets.tres"
)
#var metronome_click: AudioStream = preload(AudioConsts.METRONOME_CLICK)
var time_of_next_click: float = 0.0
var bpm: float
var song_time: float = 0.0

#var audio_stream: AudioStream = preload(AudioConsts.SHADERTOY_MUSIC_TRACK_EXPERIMENT_WAV)
#var audio_stream: AudioStream = preload(AudioConsts.HELLION_WAV)
#var audio_stream: AudioStream = preload(AudioConsts.SNUFFY)
#var input_stream: AudioStreamMicrophone = AudioStreamMicrophone.new()
#var wav_stream: AudioStreamWAV = preload("res://Resources/Audio/Cache/cached_midi.wav")


func _ready() -> void:
    var bus_index: int = AudioBus.get_bus_index(AudioBus.BUS.MUSIC)
    var effect: AudioEffectSpectrumAnalyzer = AudioEffectSpectrumAnalyzer.new()
    effect.fft_size = AudioEffectSpectrumAnalyzer.FFTSize.FFT_SIZE_2048
    AudioEffectManager.add_effect(bus_index, effect)
    var effect_index: int = AudioServer.get_bus_effect_count(bus_index) - 1
    spectrum_analyzer_instance = AudioServer.get_bus_effect_instance(bus_index, effect_index)
    #derive_bpm()
    #load_custom_onsets()

    #TODO: figure out where to actually have music play in the game, currently
    # juggling too many locations where music is tested for playback
    # especially PitchDimension scene with all the caching and shit

    #AudioPoolManager.play_music(wav_stream)

    #AudioPoolManager.play_music(audio_stream)
    #AudioPoolManager.play_input(input_stream)


func derive_bpm() -> void:
    #TODO: we no longer use global paths, fix this
    #var wav_fs_path: String = ProjectSettings.globalize_path(song)
    #bpm = rust_util.detect_bpm(wav_fs_path)
    print("aubio derived bpm is:", bpm)


static var custom_onsets_flat_buffer: PackedVector4Array


func load_custom_onsets() -> void:
    custom_onsets_flat_buffer.clear()
    var uki: PackedFloat32Array = custom_onset_data.uki
    var shizumi: PackedFloat32Array = custom_onset_data.shizumi
    var total_onsets: float = min(uki.size(), shizumi.size()) / 2
    for i: int in range(int(total_onsets)):
        var u_start: float = uki[i * 2]
        var u_end: float = uki[i * 2 + 1]
        var s_start: float = shizumi[i * 2]
        var s_end: float = shizumi[i * 2 + 1]
        var uki_flat: Vector2 = Vector2(u_start, u_end)
        var shizumi_flat: Vector2 = Vector2(s_start, s_end)
        var uki_shizumi_flat: Vector4 = Vector4(
            uki_flat.x, uki_flat.y, shizumi_flat.x, shizumi_flat.y
        )
        custom_onsets_flat_buffer.append(uki_shizumi_flat)


func debug_custom_onsets(delta: float) -> void:
    var uki_onset_index: int = 0
    var shizumi_onset_index: int = 0
    song_time += delta
    while uki_onset_index < custom_onsets_flat_buffer.size():
        var next_uki_onset: float = custom_onsets_flat_buffer[uki_onset_index].x
        if song_time < next_uki_onset:
            break
        #AudioPoolManager.play_sfx(metronome_click)
        uki_onset_index += 1

    while shizumi_onset_index < custom_onsets_flat_buffer.size():
        var next_j_start: float = custom_onsets_flat_buffer[shizumi_onset_index].z
        if song_time < next_j_start:
            break
        #AudioPoolManager.play_sfx(metronome_click)
        shizumi_onset_index += 1


#TODO: this identical to ManualRhythmOnsetRecorder._debug_keys()
func debug_custom_onsets_ASCII(delta: float) -> void:
    var prev_time: float = song_time
    song_time += delta
    var f_char: String = " "
    var j_char: String = " "
    var f_press_fmt: String = ""
    var f_rel_fmt: String = ""
    var j_press_fmt: String = ""
    var j_rel_fmt: String = ""
    for v: Vector4 in custom_onsets_flat_buffer:
        var u_start: float = v.x
        var u_end: float = v.y
        var s_start: float = v.z
        var s_end: float = v.w
        if prev_time < u_start and song_time >= u_start:
            f_char = "F"
            f_press_fmt = "F_PRS:[%.3f,      ]" % u_start
        if prev_time < u_end and song_time >= u_end:
            f_rel_fmt = "F_REL:[%.3f, %.3f]" % [u_start, u_end]
        if prev_time < s_start and song_time >= s_start:
            j_char = "J"
            j_press_fmt = "J_PRS:[%.3f,      ]" % s_start

        if prev_time < s_end and song_time >= s_end:
            j_rel_fmt = "J_REL:[%.3f, %.3f]" % [s_start, s_end]
    var event_body: String = f_press_fmt + f_rel_fmt + j_press_fmt + j_rel_fmt
    var status_body: String = "[%s] [%s]" % [f_char, j_char]
    if event_body != "":
        status_body += "   " + event_body
        print(status_body)


func _process(_delta: float) -> void:
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
    var _time_since_previous_onset: float = time_since_previous_onset(current_playback_time)
    onset_intervals_history_buffer.append(_time_since_previous_onset)
    if onset_intervals_history_buffer.size() > ONSETS_PER_MINUTE_HISTORY_BUFFER_MAXIMUM_SIZE:
        onset_intervals_history_buffer.remove_at(0)

    onset_detected.emit(current_playback_time)
    onset_event_counter += 1
    time_of_previous_onset = current_playback_time


func time_since_previous_onset(current_playback_time: float) -> float:
    var _time_since_previous_onset: float = 1e6  #???? 0.0
    if self.time_of_previous_onset > 0.0:
        _time_since_previous_onset = current_playback_time - self.time_of_previous_onset
    return _time_since_previous_onset


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
    from_hz: float, to_hz: float, _previous_smooth_energy: float
) -> float:
    var linear_average: float = _compute_linear_average_for_frequency_range(from_hz, to_hz)
    var normalized: float = _compute_normalized_energy_from_linear_magnitude(linear_average)
    return _compute_smooth_energy(_previous_smooth_energy, normalized)


func _compute_linear_average_for_frequency_range(from_hz: float, to_hz: float) -> float:
    var stereo_magnitude: Vector2 = spectrum_analyzer_instance.get_magnitude_for_frequency_range(
        from_hz, to_hz, AudioEffectSpectrumAnalyzerInstance.MagnitudeMode.MAGNITUDE_AVERAGE
    )
    return (stereo_magnitude.x + stereo_magnitude.y) * 0.5


func _compute_normalized_energy_from_linear_magnitude(linear_magnitude: float) -> float:
    var db: float = linear_to_db(linear_magnitude)
    return clamp(
        (db - MDN_MIN_AUDIO_DECIBEL) / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL), 0.0, 1.0
    )


func _compute_smooth_energy(_previous_smooth_energy: float, new_normalized_energy: float) -> float:
    return MDN_SMOOTHING * _previous_smooth_energy + (1.0 - MDN_SMOOTHING) * new_normalized_energy
