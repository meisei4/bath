extends Node
class_name ManualRhythmOnsetRecorder

const SAVE_PATH: String = "res://Resources/Audio/CustomOnsets/custom_onsets.tres"

var key_f_presses: PackedFloat32Array
var key_j_presses: PackedFloat32Array

var f_press_time: float = -1.0
var j_press_time: float = -1.0
var f_release_time: float = -1.0
var j_release_time: float = -1.0


func _ready() -> void:
    key_f_presses.clear()
    key_j_presses.clear()


func _process(delta: float) -> void:
    MusicDimensionsManager.song_time += delta
    _handle_presses()
    _handle_releases()
    _debug_keys(f_press_time, f_release_time, j_press_time, j_release_time)


func _handle_presses() -> void:
    if Input.is_action_just_pressed("F"):
        f_press_time = MusicDimensionsManager.song_time
    if Input.is_action_just_pressed("J"):
        j_press_time = MusicDimensionsManager.song_time


func _handle_releases() -> void:
    if Input.is_action_just_released("F"):
        f_release_time = MusicDimensionsManager.song_time
        key_f_presses.push_back(f_press_time)
        key_f_presses.push_back(f_release_time)

    if Input.is_action_just_released("J"):
        j_release_time = MusicDimensionsManager.song_time
        key_j_presses.push_back(j_press_time)
        key_j_presses.push_back(j_release_time)


func _debug_keys(
    f_press_time: float, f_release_time: float, j_press_time: float, j_release_time: float
) -> void:
    var f_char: String = " "
    var j_char: String = " "
    var f_press_fmt: String = ""
    var f_rel_fmt: String = ""
    var j_press_fmt: String = ""
    var j_rel_fmt: String = ""

    if Input.is_action_just_pressed("F"):
        f_char = "F"
        f_press_fmt = "F_PRS:[%.3f,      ]" % f_press_time
    elif Input.is_action_pressed("F"):
        f_char = "f"
    if Input.is_action_just_pressed("J"):
        j_char = "J"
        j_press_fmt = "J_PRS:[%.3f,      ]" % j_press_time
    elif Input.is_action_pressed("J"):
        j_char = "j"

    if Input.is_action_just_released("F"):
        f_rel_fmt = "F_REL:[%.3f, %.3f]" % [f_press_time, f_release_time]
    if Input.is_action_just_released("J"):
        j_rel_fmt = "J_REL:[%.3f, %.3f]" % [j_press_time, j_release_time]

    var event_body: String = f_press_fmt + f_rel_fmt + j_press_fmt + j_rel_fmt
    var status_body: String = "[%s] [%s]" % [f_char, j_char]
    if event_body != "":
        status_body += "   " + event_body
    print(status_body)


func _exit_tree() -> void:
    _save_onsets()


func _save_onsets() -> void:
    var onset_data: RhythmOnsetData = RhythmOnsetData.new()
    onset_data.uki = key_f_presses
    onset_data.shizumi = key_j_presses
    var err: Error = ResourceSaver.save(onset_data, SAVE_PATH)
