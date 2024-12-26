# AudioZone.gd
extends AudioStreamPlayer2D
class_name AudioZone

@export var max_effect_distance: float = 300.0
@export var effects_enabled: bool = true
@export var effect_min_value: float = 0.0
@export var effect_max_value: float = 1.0
@export var audio_bus: AudioBus.BUS = AudioBus.BUS.MUSIC  # Default audio bus

var viewer: Node = null  # Node used to calculate the listener's position

# Flags to control dynamic effects
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

    if not self.stream:
        push_error("No audio stream assigned to AudioZone.")
        return
    print("AudioZone stream assigned: ", self.stream.resource_path)

    # Configure properties
    self.audio_bus = AudioBus.BUS.MUSIC
    print("AudioZone bus set to: ", audio_bus)

    # Initialize AudioEffects via AudioEffects singleton
    if effects_enabled:
        AudioEffects.add_distortion(audio_bus, AudioEffects.DEFAULT_DISTORTION)
        AudioEffects.add_reverb(audio_bus, AudioEffects.DEFAULT_REVERB)
        AudioEffects.set_pitch_shift(audio_bus, AudioEffects.DEFAULT_PITCH_SHIFT["pitch_scale"])
        print("Initial AudioEffects added.")

    # Start playback if stream is valid
    if self.stream:
        self.play()
        print("AudioZone playback started.")

func _process(delta: float) -> void:
    if not viewer:
        return  # No viewer, skip processing

    # Calculate normalized distance
    var distance = viewer.global_position.distance_to(self.global_position)
    var normalized_distance = clamp(1.0 - (distance / max_effect_distance), 0.0, 1.0)
    var effect_strength = lerp(effect_min_value, effect_max_value, normalized_distance)

    # Apply custom volume effect
    apply_custom_volume(effect_strength)

    # Adjust AudioEffects based on effect_strength if enabled
    if effects_enabled:
        if dynamic_distortion_enabled:
            adjust_distortion(effect_strength)
        if dynamic_reverb_enabled:
            adjust_reverb(effect_strength)
        if dynamic_pitch_shift_enabled:
            adjust_pitch_shift(effect_strength)

func initialize_viewer() -> void:
    # Try to find the player in the "player" group
    var players = get_tree().get_nodes_in_group("player")
    if players.size() > 0:
        viewer = players[0]
        print("Player node found: ", viewer.name)
        return

    # If no player, try to find the current camera
    var current_camera = get_viewport().get_camera_2d()
    if current_camera:
        viewer = current_camera
        print("Using camera as viewer: ", current_camera.name)
        return

    # If neither exists, viewer will remain null
    push_warning("No viewer found. AudioZone effects will not be applied.")

func apply_custom_volume(effect_strength: float) -> void:
    # Adjust the volume based on proximity
    # Increased the range for louder volume when close
    self.volume_db = lerp(-20.0, 0.0, effect_strength)
    print("Volume adjusted to: ", self.volume_db)

# Adjust AudioEffects parameters based on effect_strength
func adjust_distortion(effect_strength: float) -> void:
    var config = {
        "drive": lerp(0.0, AudioEffects.DEFAULT_DISTORTION["drive"], effect_strength),
        "pre_gain_db": lerp(0.0, AudioEffects.DEFAULT_DISTORTION["pre_gain_db"], effect_strength),
        "post_gain_db": lerp(0.0, AudioEffects.DEFAULT_DISTORTION["post_gain_db"], effect_strength)
    }
    AudioEffects.update_distortion(audio_bus, config)
    print("Distortion adjusted with config: ", config)

func adjust_reverb(effect_strength: float) -> void:
    var config = {
        "wet": lerp(0.0, AudioEffects.DEFAULT_REVERB["wet"], effect_strength),
        "room_size": lerp(0.0, AudioEffects.DEFAULT_REVERB["room_size"], effect_strength),
        "damping": lerp(0.0, AudioEffects.DEFAULT_REVERB["damping"], effect_strength)
    }
    AudioEffects.update_reverb(audio_bus, config)
    print("Reverb adjusted with config: ", config)

func adjust_pitch_shift(effect_strength: float) -> void:
    current_pitch = lerp(0.5, 2.0, effect_strength)  # Pitch shifts between 0.5 and 2.0
    AudioEffects.set_pitch_shift(audio_bus, current_pitch)
    print("Pitch shift adjusted to: ", current_pitch)

# Methods to toggle dynamic effects
func enable_dynamic_distortion() -> void:
    dynamic_distortion_enabled = true
    print("Dynamic distortion enabled.")

func disable_dynamic_distortion() -> void:
    dynamic_distortion_enabled = false
    print("Dynamic distortion disabled.")

func enable_dynamic_reverb() -> void:
    dynamic_reverb_enabled = true
    print("Dynamic reverb enabled.")

func disable_dynamic_reverb() -> void:
    dynamic_reverb_enabled = false
    print("Dynamic reverb disabled.")

func enable_dynamic_pitch_shift() -> void:
    dynamic_pitch_shift_enabled = true
    print("Dynamic pitch shift enabled.")

func disable_dynamic_pitch_shift() -> void:
    dynamic_pitch_shift_enabled = false
    print("Dynamic pitch shift disabled.")
