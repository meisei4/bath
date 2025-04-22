extends Mechanic
class_name Jump

const INITIAL_JUMP_VELOCITY: float = 10.0
const FORWARD_SPEED: float = 5.0
const SPRITE_SCALE_AT_MAX_ALTITUDE: float = 2.0
const OVERRIDE_GRAVITY: float = 0.0  #TODO: these things will be moved to a resource for controlling spacetime context

var gravity_value: float
var vertical_speed: float
var vertical_position: float

enum JumpPhase { GROUNDED, ASCENDING, DESCENDING }
var current_phase: JumpPhase


func _ready() -> void:
    vertical_position = 0.0
    vertical_speed = 0.0
    _set_phase(JumpPhase.GROUNDED)
    apply_mechanic_animation_shader("res://Resources/Shaders/MechanicAnimations/jump_trig.gdshader")
    MechanicManager.jump.connect(_on_jump)
    gravity_value = OVERRIDE_GRAVITY if OVERRIDE_GRAVITY > 0.0 else SpacetimeContext.GRAVITY


func process_input(frame_delta: float) -> void:
    var time_scaled_delta: float = SpacetimeContext.apply_time_scale(frame_delta)
    _apply_gravity_and_drag(time_scaled_delta)
    _update_altitude(time_scaled_delta)
    if _should_land():
        _handle_landing()
    if is_airborne() and FORWARD_SPEED != 0.0:
        _apply_forward_movement(time_scaled_delta)


func _apply_gravity_and_drag(time_scaled_delta: float) -> void:
    if is_airborne():
        vertical_speed = vertical_speed - (gravity_value * time_scaled_delta)
        vertical_speed = SpacetimeContext.apply_universal_drag(vertical_speed, time_scaled_delta)


func _update_altitude(time_scaled_delta: float) -> void:
    vertical_position += vertical_speed * time_scaled_delta
    if vertical_speed > 0.0:
        _set_phase(JumpPhase.ASCENDING)
    elif vertical_speed < 0.0 and vertical_position > 0.0:
        _set_phase(JumpPhase.DESCENDING)


func _apply_forward_movement(time_scaled_delta: float) -> void:
    var forward_movement_world_units: float = FORWARD_SPEED * time_scaled_delta
    var forward_movement_pixel_units: float = SpacetimeContext.to_physical_space(
        forward_movement_world_units
    )
    character.position.y = character.position.y - forward_movement_pixel_units


func process_visual_illusion(_frame_delta: float) -> void:
    var sprite_node: Sprite2D = get_sprite_for_visual_illusion()
    var vertical_offset_pixels: float = SpacetimeContext.to_physical_space(vertical_position)
    sprite_node.position.y = -vertical_offset_pixels
    var maximum_jump_height: float = _calculate_parabolic_max_altitude()
    var altitude_normal: float = _compute_altitude_normal_in_jump_parabola(
        vertical_position, maximum_jump_height
    )
    sprite_node.material.set_shader_parameter("ascending", is_ascending())
    sprite_node.material.set_shader_parameter("altitude_normal", altitude_normal)
    _update_sprite_scale(sprite_node, altitude_normal)


func _calculate_parabolic_max_altitude() -> float:
    if gravity_value > 0.0:
        var squared_initial_velocity: float = INITIAL_JUMP_VELOCITY * INITIAL_JUMP_VELOCITY
        var denominator: float = 2.0 * gravity_value
        return squared_initial_velocity / denominator
    else:
        return 0.0


func _compute_altitude_normal_in_jump_parabola(
    current_height: float, maximum_height: float
) -> float:
    if maximum_height > 0.0:
        var raw_ratio: float = current_height / maximum_height
        return clamp(raw_ratio, 0.0, 1.0)
    else:
        return 0.0


func _update_sprite_scale(sprite_node: Sprite2D, altitude_location: float) -> void:
    var scale_minimum: float = 1.0
    #var scale_multiplier: float = lerp(1.0, SPRITE_SCALE_AT_MAX_ALTITUDE, altitude_location)
    #TODO: below is an explicit lerp function
    var scale_multiplier: float = (
        scale_minimum + ((SPRITE_SCALE_AT_MAX_ALTITUDE - scale_minimum) * altitude_location)
    )
    sprite_node.scale = Vector2.ONE * scale_multiplier


func process_collision_shape(_delta: float) -> void:
    var collision_shape: CollisionShape2D = get_collision_object_for_processing()
    if _is_grounded():
        collision_shape.disabled = false  #TODO: lmao double negatives
    else:
        collision_shape.disabled = true


func _on_jump() -> void:
    if !is_airborne():
        vertical_speed = INITIAL_JUMP_VELOCITY
        _set_phase(JumpPhase.ASCENDING)


func _is_grounded() -> bool:
    return current_phase == JumpPhase.GROUNDED


func _is_vertically_idle() -> bool:
    return vertical_speed == 0.0


func is_ascending() -> bool:
    return current_phase == JumpPhase.ASCENDING


func is_descending() -> bool:
    return current_phase == JumpPhase.DESCENDING


func is_airborne() -> bool:
    return current_phase != JumpPhase.GROUNDED


func _should_land() -> bool:
    return is_descending() and vertical_position <= 0.0


func _handle_landing() -> void:
    vertical_position = 0.0
    vertical_speed = 0.0
    _set_phase(JumpPhase.GROUNDED)


func _set_phase(new_phase: JumpPhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase
