extends Mechanic
class_name Swim

enum DivePhase { LEVEL, ASCENDING, DIVING }  #TODO: i dont want to add an APEX_FLOAT phase but maybe...
var current_phase: DivePhase

const LEVEL_DEPTH: float = 0.0
const MAX_DIVE_DEPTH: float = -1.0  # deepest it ever gets should only ever be 1
const DEPTH_SPEED: float = 4.0

var current_depth_position: float = LEVEL_DEPTH
var target_depth_position: float = LEVEL_DEPTH


func _ready() -> void:
    mechanic_shader = preload("res://Resources/Shaders/MechanicAnimations/swim.gdshader")
    current_depth_position = LEVEL_DEPTH
    _set_phase(DivePhase.LEVEL)
    #TODO: figure out how to make this shaders default effect be at scale = 1 and absolutely no glitch snapping of the sprite when jumping finishes
    #MusicDimensionsManager.rhythm_indicator.connect(_on_rhythm_indicator)


func _on_rhythm_indicator(beat_index: int, bar_index: int, beats_per_minute: float) -> void:
    #print("Swim got beat:", beat_index, "→ target_depth=", target_depth_position)
    if beat_index % MusicDimensionsManager.time_signature == 0:
        target_depth_position = MAX_DIVE_DEPTH
    else:
        target_depth_position = LEVEL_DEPTH


var debug_autopogo: bool = true  # turn off → normal rhythm behaviour
const _DEBUG_PERIOD: float = 1.0
var _debug_clock: float = 0.0


func _process(delta: float) -> void:
    if !debug_autopogo:
        return
    _debug_clock += delta
    if _debug_clock >= _DEBUG_PERIOD:
        _debug_clock -= _DEBUG_PERIOD
        target_depth_position = (
            MAX_DIVE_DEPTH if target_depth_position == LEVEL_DEPTH else LEVEL_DEPTH
        )


func process_input(delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _update_depth(time_scaled_delta)


func _update_depth(delta: float) -> void:
    var step: float = DEPTH_SPEED * delta
    current_depth_position = clamp(
        move_toward(current_depth_position, target_depth_position, step),
        MAX_DIVE_DEPTH,
        LEVEL_DEPTH
    )

    if target_depth_position == LEVEL_DEPTH:
        if !is_ascending() and current_depth_position > MAX_DIVE_DEPTH:
            _set_phase(DivePhase.ASCENDING)
    else:
        if !is_diving() and current_depth_position < LEVEL_DEPTH:
            _set_phase(DivePhase.DIVING)

    if abs(current_depth_position - LEVEL_DEPTH) == 0.0:
        _set_phase(DivePhase.LEVEL)


func process_visual_illusion(_frame_delta: float) -> void:
    var sprite_node: Sprite2D = get_sprite()
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(current_depth_position)
    sprite_node.position.y = -vertical_offset_pixels
    sprite_node.material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_node.material.set_shader_parameter("ascending", is_ascending())
    sprite_node.material.set_shader_parameter("depth_normal", _depth_normal())

    _update_sprite_scale(sprite_node, _depth_normal(), _frame_delta)

    ComputeShaderSignalManager.visual_illusion_updated.emit(
        sprite_texture_index,
        sprite_node.global_position,
        (sprite_node.texture.get_size() * 0.5) * sprite_node.scale,
        _depth_normal(),
        1.0 if is_ascending() else 0.0
    )


func _depth_normal() -> float:
    var depth: float = clamp(-current_depth_position / abs(MAX_DIVE_DEPTH), 0.0, 1.0)
    return pow(depth, 1.6)


var current_scale: float = 1.0
var target_scale: float = 1.0
var scale_velocity: float = 0.0
var scale_smoothing_time: float = 0.15


func _update_sprite_scale(sprite: Sprite2D, depth_normal: float, frame_delta: float ) -> void:
    var scale_min: float = 0.5
    var scale_max: float = 1.0
    target_scale = lerp(scale_max, scale_min, depth_normal)

    var result: Vector2 = smooth_damp(
        current_scale, target_scale, scale_velocity, scale_smoothing_time, frame_delta
    )
    current_scale = result.x  # new value
    scale_velocity = result.y  # new velocity (for next frame)

    sprite.scale = Vector2.ONE * current_scale


func smooth_damp(
    current: float, target: float, velocity: float, smooth_time: float, delta: float
) -> Vector2:
    var omega: float = 2.0 / smooth_time
    var x: float = omega * delta
    var exp_factor: float = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x)

    var change: float = current - target
    var temp: float = (velocity + omega * change) * delta
    velocity = (velocity - omega * temp) * exp_factor
    var output: float = target + (change + temp) * exp_factor
    return Vector2(output, velocity)


func is_diving() -> bool:
    return current_phase == DivePhase.DIVING


func is_ascending() -> bool:
    return current_phase == DivePhase.ASCENDING


func is_level() -> bool:
    return current_phase == DivePhase.LEVEL


func _set_phase(new_phase: DivePhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase
