extends Mechanic
class_name Jump

@export var PARAMETERS: JumpData

enum JumpPhase { GROUNDED, ASCENDING, DESCENDING }  #TODO: i dont want to add an APEX_FLOAT phase but maybe...
var current_phase: JumpPhase
var vertical_speed: float
var vertical_position: float


func _ready() -> void:
    mechanic_shader = preload(ResourcePaths.JUMP_TRIG_SHADER)
    if PARAMETERS == null:
        PARAMETERS = JumpData.new()
    vertical_position = 0.0
    vertical_speed = 0.0
    _set_phase(JumpPhase.GROUNDED)


func process_input(frame_delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(frame_delta)
    _apply_gravity_and_drag(time_scaled_delta)
    _update_altitude(time_scaled_delta)
    if _should_land():
        _handle_landing()
    if is_airborne() and PARAMETERS.FORWARD_SPEED != 0.0:
        _apply_forward_movement(time_scaled_delta)


func _apply_gravity_and_drag(time_scaled_delta: float) -> void:
    if is_airborne():
        var gravity: float = _get_effective_gravity()
        vertical_speed -= gravity * time_scaled_delta
        vertical_speed = SpacetimeManager.apply_universal_drag(vertical_speed, time_scaled_delta)


func _update_altitude(time_scaled_delta: float) -> void:
    vertical_position += vertical_speed * time_scaled_delta
    if vertical_speed > 0.0:
        _set_phase(JumpPhase.ASCENDING)
    elif vertical_speed < 0.0 and vertical_position > 0.0:
        _set_phase(JumpPhase.DESCENDING)


func _apply_forward_movement(time_scaled_delta: float) -> void:
    var forward_movement_world_units: float = PARAMETERS.FORWARD_SPEED * time_scaled_delta
    var forward_movement_pixel_units: float = SpacetimeManager.to_physical_space(
        forward_movement_world_units
    )
    character_body.position.y = character_body.position.y - forward_movement_pixel_units


func process_visual_illusion(_frame_delta: float) -> void:
    var sprite_node: Sprite2D = super.get_sprite()  #TODO: there is now an active_sprite attribute in the mechanics....
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(vertical_position)
    sprite_node.position.y = roundi(-vertical_offset_pixels)
    var max_altitude: float = _max_altitude()
    var altitude_normal: float = _compute_altitude_normal_in_jump_parabola(
        vertical_position, max_altitude
    )
    #_update_sprite_scale_continious(sprite_node, altitude_normal)
    _update_sprite_scale_discrete(sprite_node, altitude_normal)
    sprite_node.material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_node.material.set_shader_parameter("iResolution", sprite_node.texture.get_size())
    var _is_ascending: bool = is_ascending()
    sprite_node.material.set_shader_parameter("ascending", _is_ascending)
    var sprite_height: float = sprite_node.texture.get_size().y
    #TODO: quantize the altitude normal is super important to study for later as it controls exactly how many
    # unique warped sprite frames can exist in the animation
    #TODO: the biggest thing left is quantizing such that we can control a hand-drawn looking pixel perfect tilt animation
    altitude_normal = roundf(altitude_normal * sprite_height) / (sprite_height)  #* 2.0)
    sprite_node.material.set_shader_parameter("altitude_normal", altitude_normal)
    MechanicsManager.visual_illusion_updated.emit(
        sprite_texture_index,
        sprite_node.global_position,
        sprite_node.scale,
        altitude_normal,
        1.0 if _is_ascending else 0.0
    )


func _max_altitude() -> float:
    if _get_effective_gravity() > 0.0:
        var squared_initial_velocity: float = (
            PARAMETERS.INITIAL_JUMP_VELOCITY * PARAMETERS.INITIAL_JUMP_VELOCITY
        )
        var denominator: float = 2.0 * _get_effective_gravity()
        return squared_initial_velocity / denominator
    else:
        return 0.0


func _compute_altitude_normal_in_jump_parabola(
    _vertical_position: float, max_altitude: float
) -> float:
    if max_altitude == 0.0:
        return 0.0
    else:
        var altitude_normal: float = _vertical_position / max_altitude
        return clamp(altitude_normal, 0.0, 1.0)


func _update_sprite_scale_continious(sprite_node: Sprite2D, _altitude_normal: float) -> void:
    var scale_minimum: float = PARAMETERS.SPRITE_SCALE_AT_MIN_ALTITUDE
    var scale_maximum: float = PARAMETERS.SPRITE_SCALE_AT_MAX_ALTITUDE
    var scale_multiplier: float = scale_minimum + (scale_maximum - scale_minimum) * _altitude_normal
    sprite_node.scale = Vector2.ONE * scale_multiplier


func _eased_phase(_altitude_normal: float) -> float:
    const EASE_EXP: float = 2.0
    return pow(_altitude_normal, EASE_EXP)


func _update_sprite_scale_discrete(sprite_node: Sprite2D, _altitude_normal: float) -> void:
    var base_width_i: int = int(sprite_node.texture.get_size().x)
    var base_height_i: int = int(sprite_node.texture.get_size().y)
    var scale_minimum_f: float = PARAMETERS.SPRITE_SCALE_AT_MIN_ALTITUDE
    var scale_maximum_f: float = PARAMETERS.SPRITE_SCALE_AT_MAX_ALTITUDE
    var eased_altitude_normal: float = _eased_phase(_altitude_normal)
    #var interpolated_scale_f: float = lerp(scale_minimum_f, scale_maximum_f, altitude_location)
    #TODO: below is an explicit lerp function
    var interpolated_scale_f: float = (
        scale_minimum_f + (scale_maximum_f - scale_minimum_f) * eased_altitude_normal
    )
    # Example (base 16×24), min scale = 1.0, max scale = 3.0
    # altitude_normal | interp_scale_f |  gcd_uniform  | scale_mult_f | final dimensions
    # -----------------------------------------------------------------------------------
    # 0.00            | 1.0            | 8             | 1.0          | (16, 24)
    # 0.25            | 1.5            | 12            | 1.5          | (24, 36)
    # 0.50            | 2.0            | 16            | 2.0          | (32, 48)
    # 0.75            | 2.5            | 20            | 2.5          | (40, 60)
    # 1.00            | 3.0            | 24            | 3.0          | (48, 72)
    var temp_a: int = base_width_i
    var temp_b: int = base_height_i
    while temp_b != 0:
        var remainder: int = temp_a % temp_b
        temp_a = temp_b
        temp_b = remainder
    var greatest_common_divisor_uniform: int = temp_a
    var steps_i: int = greatest_common_divisor_uniform
    var quantized_steps_i: int = roundi(interpolated_scale_f * steps_i)
    var scale_multiplier_f: float = quantized_steps_i / float(steps_i)
    sprite_node.scale = Vector2.ONE * scale_multiplier_f


func process_collision_shape(_delta: float) -> void:
    var collision_shape: CollisionShape2D = super.get_collision_shape()
    if _is_grounded():
        collision_shape.disabled = false  #TODO: lmao double negatives
    else:
        collision_shape.disabled = true


func _on_jump() -> void:
    if !is_airborne():
        vertical_speed = PARAMETERS.INITIAL_JUMP_VELOCITY
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


func _get_effective_gravity() -> float:
    return (
        PARAMETERS.OVERRIDE_GRAVITY
        if PARAMETERS.OVERRIDE_GRAVITY > 0.0
        else SpacetimeManager.GRAVITY
    )
