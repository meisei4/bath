extends AudioStreamPlayer2D
class_name AudioZone

@export var max_effect_distance: float = 300.0
@export var effects_enabled: bool = true
@export var effect_min_value: float = 0.0
@export var effect_max_value: float = 1.0

var viewer: Node = null  # Node used to calculate the listener's position


func _ready() -> void:
    initialize_viewer()

    if not self.stream:
        push_error("No audio stream assigned to AudioZone.")
        return
    print("AudioZone stream assigned: ", self.stream.resource_path)

    # Configure properties
    self.max_distance = max_effect_distance
    self.attenuation = 1.0
    self.bus = "Master"
    print("AudioZone properties set.")

    # Start playback if stream is valid
    if self.stream:
        self.play()
        print("AudioZone playback started.")


func _process(delta: float) -> void:
    if not viewer:
        return  # No viewer, skip processing

    # Calculate normalized distance
    var distance = clamp(
        viewer.global_position.distance_to(self.global_position), 0, max_effect_distance
    )
    var normalized_distance = 1.0 - (distance / max_effect_distance)
    var effect_strength = lerp(effect_min_value, effect_max_value, normalized_distance)

    # Apply effects if enabled
    if effects_enabled:
        apply_custom_effect(effect_strength)


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
        print("Using camera as viewer: ", viewer.name)
        return

    # If neither exists, viewer will remain null
    push_warning("No viewer found. AudioZone effects will not be applied.")


func apply_custom_effect(effect_strength: float) -> void:
    self.volume_db = lerp(-80.0, 0.0, effect_strength)
    print("Volume adjusted to: ", self.volume_db)
