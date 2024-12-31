extends AudioStreamPlayer2D
class_name AudioZone

@export var max_effect_distance: float = 300.0
@export var effects_enabled: bool = true
@export var effect_min_value: float = 0.0
@export var effect_max_value: float = 1.0
@export var audio_bus: AudioBus.BUS = AudioBus.BUS.MUSIC

var viewer: Node = null

var dynamic_distortion_enabled: bool = false
var dynamic_reverb_enabled: bool = false
var dynamic_pitch_shift_enabled: bool = false

var current_pitch: float = 1.0

# TODO: godot enums suck, like they seriously suck, or else im the dumbest person on earth
#var MASTER: String = AudioBus.val(AudioBus.BUS.MASTER)
#var SFX: String = AudioBus.val(AudioBus.BUS.SFX)
#var MUSIC: String = AudioBus.val(AudioBus.BUS.MUSIC)
#const MASTER: String = "Master"
#const SFX: String = "SFX"
#const MUSIC: String = "Music"


func _ready() -> void:
    initialize_viewer()
    self.audio_bus = AudioBus.BUS.MUSIC

    if effects_enabled:
        AudioEffects.add_distortion(audio_bus, AudioEffects.DEFAULT_DISTORTION)
        AudioEffects.add_reverb(audio_bus, AudioEffects.DEFAULT_REVERB)
        AudioEffects.set_pitch_shift(audio_bus, AudioEffects.DEFAULT_PITCH_SHIFT["pitch_scale"])

    if self.stream:
        self.play()


func _process(delta: float) -> void:
    if not viewer:
        return

    var distance = viewer.global_position.distance_to(self.global_position)
    var normalized_distance = clamp(1.0 - (distance / max_effect_distance), 0.0, 1.0)
    var effect_strength = lerp(effect_min_value, effect_max_value, normalized_distance)

    apply_custom_volume(effect_strength)

    if effects_enabled:
        if dynamic_distortion_enabled:
            adjust_distortion(effect_strength)
        if dynamic_reverb_enabled:
            adjust_reverb(effect_strength)
        if dynamic_pitch_shift_enabled:
            adjust_pitch_shift(effect_strength)


func initialize_viewer() -> void:
    var players = get_tree().get_nodes_in_group("player")
    if players.size() > 0:
        viewer = players[0]
        print("Player node found: ", viewer.name)
        return

    var current_camera = get_viewport().get_camera_2d()
    if current_camera:
        viewer = current_camera
        return


func apply_custom_volume(effect_strength: float) -> void:
    self.volume_db = lerp(-20.0, 0.0, effect_strength)


func adjust_distortion(effect_strength: float) -> void:
    var config = {
        "drive": lerp(0.0, AudioEffects.DEFAULT_DISTORTION["drive"], effect_strength),
        "pre_gain_db": lerp(0.0, AudioEffects.DEFAULT_DISTORTION["pre_gain_db"], effect_strength),
        "post_gain_db": lerp(0.0, AudioEffects.DEFAULT_DISTORTION["post_gain_db"], effect_strength)
    }
    AudioEffects.update_distortion(audio_bus, config)


func adjust_reverb(effect_strength: float) -> void:
    var config = {
        "wet": lerp(0.0, AudioEffects.DEFAULT_REVERB["wet"], effect_strength),
        "room_size": lerp(0.0, AudioEffects.DEFAULT_REVERB["room_size"], effect_strength),
        "damping": lerp(0.0, AudioEffects.DEFAULT_REVERB["damping"], effect_strength)
    }
    AudioEffects.update_reverb(audio_bus, config)


func adjust_pitch_shift(effect_strength: float) -> void:
    current_pitch = lerp(0.5, 2.0, effect_strength)  # Pitch shifts between 0.5 and 2.0
    AudioEffects.set_pitch_shift(audio_bus, current_pitch)


func enable_dynamic_distortion() -> void:
    dynamic_distortion_enabled = true


func disable_dynamic_distortion() -> void:
    dynamic_distortion_enabled = false


func enable_dynamic_reverb() -> void:
    dynamic_reverb_enabled = true


func disable_dynamic_reverb() -> void:
    dynamic_reverb_enabled = false


func enable_dynamic_pitch_shift() -> void:
    dynamic_pitch_shift_enabled = true


func disable_dynamic_pitch_shift() -> void:
    dynamic_pitch_shift_enabled = false
