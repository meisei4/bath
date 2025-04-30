extends Mechanic
class_name Swim

const SWIM_SHADER: Shader = preload("res://Resources/Shaders/MechanicAnimations/swim.gdshader")

enum DivePhase { LEVEL, ASCENDING, DIVING }  #TODO: i dont want to add an APEX_FLOAT phase but maybe...
var current_phase: DivePhase
var current_depth_position: float = 0.0
var target_depth_position: float = 0.0
var depth_smoothing_speed: float = 5.0  # how quickly we interpolate depth
var _is_surface_override_in_progress: bool = false


func _ready() -> void:
    current_depth_position = 1.0
    _set_phase(DivePhase.LEVEL)
    apply_mechanic_animation_shader(SWIM_SHADER)
    MusicDimensionsManager.rhythm_indicator.connect(_on_rhythm_indicator)
    MechanicManager.ascend_to_surface.connect(_on_ascend_to_surface)
    MechanicManager.resume_swim.connect(_on_resume_swim)


func _on_ascend_to_surface() -> void:
    MusicDimensionsManager.rhythm_indicator.disconnect(_on_rhythm_indicator)
    _is_surface_override_in_progress = true
    target_depth_position = 1.0
    _set_phase(DivePhase.ASCENDING)


func _on_resume_swim() -> void:
    apply_mechanic_animation_shader(SWIM_SHADER)
    current_depth_position = 1.0
    _set_phase(DivePhase.LEVEL)
    visuals_enabled = true
    MusicDimensionsManager.rhythm_indicator.connect(_on_rhythm_indicator)


func _complete_surface_override() -> void:
    visuals_enabled = false
    _is_surface_override_in_progress = false
    MechanicManager.jump.emit()


func _on_rhythm_indicator(beat_index: int, bar_index: int, beats_per_minute: float) -> void:
    print("Swim got beat:", beat_index, "→ target_depth=", target_depth_position)
    if beat_index % MusicDimensionsManager.time_signature == 0:
        target_depth_position = 0.0
        _set_phase(DivePhase.DIVING)
    else:
        target_depth_position = 1.0
        _set_phase(DivePhase.ASCENDING)


func _process(delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _update_depth(time_scaled_delta)


func _update_depth(time_scaled_delta: float) -> void:
    var factor: float = clamp(time_scaled_delta * depth_smoothing_speed, 0.0, 1.0)
    current_depth_position = clamp(
        lerp(current_depth_position, target_depth_position, factor), 0.0, 1.0
    )

    if _is_surface_override_in_progress:
        if abs(current_depth_position - 1.0) < 0.01:
            _complete_surface_override()


func process_visual_illusion(_frame_delta: float) -> void:
    var sprite_node: Sprite2D = get_sprite_for_visual_illusion()
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(current_depth_position)
    sprite_node.position.y = -vertical_offset_pixels
    sprite_node.material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_node.material.set_shader_parameter("ascending", is_ascending())
    var depth_normal: float = current_depth_position  #TODO: get some sort of normal???
    sprite_node.material.set_shader_parameter("depth_normal", depth_normal)
    _update_sprite_scale(sprite_node, depth_normal)
    ComputeShaderSignalManager.visual_illusion_updated.emit(
        sprite_texture_index,
        sprite_node.global_position,
        (sprite_node.texture.get_size() * 0.5) * sprite_node.scale,
        depth_normal,
        1.0 if is_ascending() else 0.0
    )


func _update_sprite_scale(sprite_node: Sprite2D, depth_location: float) -> void:
    var scale_minimum: float = 0.25
    var scale_maximum: float = 1.0
    var scale_multiplier: float = scale_minimum + (scale_maximum - scale_minimum) * depth_location
    sprite_node.scale = Vector2.ONE * scale_multiplier


func is_diving() -> bool:
    return current_phase == DivePhase.DIVING


func is_ascending() -> bool:
    return current_phase == DivePhase.ASCENDING


func is_level() -> bool:
    return current_phase == DivePhase.LEVEL


func _set_phase(new_phase: DivePhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase
