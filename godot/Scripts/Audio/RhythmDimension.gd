extends Node
class_name RhythmDimension

var metronome_click: AudioStream
var rhythm_data: RhythmData
var bpm: float
var f_onsets_flat_buffer: PackedVector2Array = PackedVector2Array()
var j_onsets_flat_buffer: PackedVector2Array = PackedVector2Array()
var f_onset_count: int = 0
var j_onset_count: int = 0

var time_of_next_click: float = 0.0
#var ogg_stream: AudioStreamOggVorbis = preload(ResourcePaths.SHADERTOY_MUSIC_EXPERIMENT_OGG)
var ogg_stream: AudioStreamWAV = preload(ResourcePaths.CACHED_WAV)


func _ready() -> void:
    if ResourceLoader.exists(ResourcePaths.CACHED_RHYTHM_DATA):
        rhythm_data = load(ResourcePaths.CACHED_RHYTHM_DATA) as RhythmData
    else:
        rhythm_data = RhythmData.new()

    if rhythm_data.bpm <= 0.0:
        #bpm = RustUtilSingleton.rust_util.detect_bpm_ogg(
        #   ResourcePaths.SHADERTOY_MUSIC_EXPERIMENT_OGG
        #)
        bpm = RustUtilSingleton.rust_util.detect_bpm_wav(ResourcePaths.CACHED_WAV)
        print("Offline BPM detection → ", bpm)
        rhythm_data.bpm = bpm
        ResourceSaver.save(rhythm_data, ResourcePaths.CACHED_RHYTHM_DATA)
    else:
        bpm = rhythm_data.bpm
        print("Using cached BPM → ", bpm)

    load_custom_onsets()
    AudioPoolManager.play_music(ogg_stream)


func load_custom_onsets() -> void:
    f_onsets_flat_buffer.clear()
    j_onsets_flat_buffer.clear()
    var uki: PackedFloat32Array = rhythm_data.uki
    var shizumi: PackedFloat32Array = rhythm_data.shizumi
    for i: int in range(0, uki.size(), 2):
        var f_press: float = uki[i]
        var f_release: float = uki[i + 1]
        f_onsets_flat_buffer.append(Vector2(f_press, f_release))
    f_onset_count = f_onsets_flat_buffer.size()
    for i: int in range(0, shizumi.size(), 2):
        var j_press: float = shizumi[i]
        var j_release: float = shizumi[i + 1]
        j_onsets_flat_buffer.append(Vector2(j_press, j_release))
    j_onset_count = j_onsets_flat_buffer.size()


func _process(delta: float) -> void:
    debug_custom_onsets_ASCII(delta)


func debug_custom_onsets_metronome_sfx(delta: float) -> void:
    var uki_onset_index: int = 0
    var shizumi_onset_index: int = 0
    MusicDimensionsManager.song_time += delta
    while uki_onset_index < f_onsets_flat_buffer.size():
        var next_uki_onset: float = f_onsets_flat_buffer[uki_onset_index].x
        if MusicDimensionsManager.song_time < next_uki_onset:
            break
        #AudioPoolManager.play_sfx(metronome_click)
        uki_onset_index += 1
    while shizumi_onset_index < j_onsets_flat_buffer.size():
        var next_j_start: float = j_onsets_flat_buffer[shizumi_onset_index].x
        if MusicDimensionsManager.song_time < next_j_start:
            break
        #AudioPoolManager.play_sfx(metronome_click)
        shizumi_onset_index += 1


func debug_custom_onsets_ASCII(delta: float) -> void:
    var prev_time: float = MusicDimensionsManager.song_time
    MusicDimensionsManager.song_time += delta
    var f_char: String = " "
    var j_char: String = " "
    var f_press_fmt: String = ""
    var f_rel_fmt: String = ""
    var j_press_fmt: String = ""
    var j_rel_fmt: String = ""
    for v: Vector2 in f_onsets_flat_buffer:
        var u_start: float = v.x
        var u_end: float = v.y
        if prev_time < u_start and MusicDimensionsManager.song_time >= u_start:
            f_char = "F"
            f_press_fmt = "F_PRS:[%.3f,      ]" % u_start
        if prev_time < u_end and MusicDimensionsManager.song_time >= u_end:
            f_rel_fmt = "F_REL:[%.3f, %.3f]" % [u_start, u_end]
    for v: Vector2 in j_onsets_flat_buffer:
        var s_start: float = v.x
        var s_end: float = v.y
        if prev_time < s_start and MusicDimensionsManager.song_time >= s_start:
            j_char = "J"
            j_press_fmt = "J_PRS:[%.3f,      ]" % s_start
        if prev_time < s_end and MusicDimensionsManager.song_time >= s_end:
            j_rel_fmt = "J_REL:[%.3f, %.3f]" % [s_start, s_end]
    var event_body: String = f_press_fmt + f_rel_fmt + j_press_fmt + j_rel_fmt
    var status_body: String = "[%s] [%s]" % [f_char, j_char]
    if event_body != "":
        status_body += "   " + event_body
        print(status_body)
