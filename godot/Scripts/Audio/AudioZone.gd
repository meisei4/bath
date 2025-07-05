extends AudioStreamPlayer2D
class_name AudioZone

var max_effect_distance: float = 300.0
var effects_enabled: bool = true
var effect_min_value: float = 0.0
var effect_max_value: float = 1.0
var audio_bus: int = AudioBus.MUSIC

var viewer: Node2D = null

var dynamic_distortion_enabled: bool = false
var dynamic_reverb_enabled: bool = false
var dynamic_pitch_shift_enabled: bool = false

var current_pitch: float = 1.0


func _ready() -> void:
    initialize_viewer()
    self.audio_bus = AudioBus.MUSIC

    if effects_enabled:
        AudioEffectManager.add_distortion(audio_bus, AudioEffectManager.DEFAULT_DISTORTION)
        AudioEffectManager.add_reverb(audio_bus, AudioEffectManager.DEFAULT_REVERB)
        AudioEffectManager.set_pitch_shift(
            audio_bus, AudioEffectManager.DEFAULT_PITCH_SHIFT["pitch_scale"]
        )

    if self.stream:
        self.play()


func _process(_delta: float) -> void:
    if not viewer:
        return

    var distance: float = viewer.global_position.distance_to(self.global_position)
    var normalized_distance: float = clamp(1.0 - (distance / max_effect_distance), 0.0, 1.0)
    var effect_strength: float = lerp(effect_min_value, effect_max_value, normalized_distance)

    apply_custom_volume(effect_strength)

    if effects_enabled:
        if dynamic_distortion_enabled:
            adjust_distortion(effect_strength)
        if dynamic_reverb_enabled:
            adjust_reverb(effect_strength)
        if dynamic_pitch_shift_enabled:
            adjust_pitch_shift(effect_strength)


func initialize_viewer() -> void:
    var players: Array[Node] = get_tree().get_nodes_in_group("player")
    if players.size() > 0:
        viewer = players[0]
        print("Player node found: ", viewer.name)
        return

    var current_camera: Camera2D = get_viewport().get_camera_2d()
    if current_camera:
        viewer = current_camera
        return


func apply_custom_volume(effect_strength: float) -> void:
    self.volume_db = lerp(-20.0, 0.0, effect_strength)


func adjust_distortion(effect_strength: float) -> void:
    var config: Dictionary = {
        "drive": lerp(0.0, AudioEffectManager.DEFAULT_DISTORTION["drive"], effect_strength),
        "pre_gain_db":
        lerp(0.0, AudioEffectManager.DEFAULT_DISTORTION["pre_gain_db"], effect_strength),
        "post_gain_db":
        lerp(0.0, AudioEffectManager.DEFAULT_DISTORTION["post_gain_db"], effect_strength)
    }
    AudioEffectManager.update_distortion(audio_bus, config)


func adjust_reverb(effect_strength: float) -> void:
    var config: Dictionary = {
        "wet": lerp(0.0, AudioEffectManager.DEFAULT_REVERB["wet"], effect_strength),
        "room_size": lerp(0.0, AudioEffectManager.DEFAULT_REVERB["room_size"], effect_strength),
        "damping": lerp(0.0, AudioEffectManager.DEFAULT_REVERB["damping"], effect_strength)
    }
    AudioEffectManager.update_reverb(audio_bus, config)


func adjust_pitch_shift(effect_strength: float) -> void:
    current_pitch = lerp(0.5, 2.0, effect_strength)  # Pitch shifts between 0.5 and 2.0
    AudioEffectManager.set_pitch_shift(audio_bus, current_pitch)


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
