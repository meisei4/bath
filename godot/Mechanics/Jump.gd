extends Mechanic
class_name Jump

@export var PARAMETERS: JumpData

enum JumpPhase { GROUNDED, ASCENDING, DESCENDING }  #TODO: i dont want to add an APEX_FLOAT phase but maybe...
var current_phase: JumpPhase
var vertical_speed: float
var vertical_position: float


func _ready() -> void:
    if PARAMETERS == null:
        PARAMETERS = JumpData.new()  # DEFAULTS
    vertical_position = 0.0
    vertical_speed = 0.0
    _set_phase(JumpPhase.GROUNDED)
    apply_mechanic_animation_shader("res://Resources/Shaders/MechanicAnimations/jump_trig.gdshader")
    MechanicManager.jump.connect(_on_jump)


func process_input(frame_delta: float) -> void:
    var time_scaled_delta: float = SpacetimeContext.apply_time_scale(frame_delta)
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
        vertical_speed = SpacetimeContext.apply_universal_drag(vertical_speed, time_scaled_delta)


func _update_altitude(time_scaled_delta: float) -> void:
    vertical_position += vertical_speed * time_scaled_delta
    if vertical_speed > 0.0:
        _set_phase(JumpPhase.ASCENDING)
    elif vertical_speed < 0.0 and vertical_position > 0.0:
        _set_phase(JumpPhase.DESCENDING)


func _apply_forward_movement(time_scaled_delta: float) -> void:
    var forward_movement_world_units: float = PARAMETERS.FORWARD_SPEED * time_scaled_delta
    var forward_movement_pixel_units: float = SpacetimeContext.to_physical_space(
        forward_movement_world_units
    )
    character.position.y = character.position.y - forward_movement_pixel_units


func process_visual_illusion(_frame_delta: float) -> void:
    var sprite_node: Sprite2D = get_sprite_for_visual_illusion()
    var vertical_offset_pixels: float = SpacetimeContext.to_physical_space(vertical_position)
    sprite_node.position.y = -vertical_offset_pixels
    var max_altitude: float = _max_altitude()
    var altitude_normal: float = _compute_altitude_normal_in_jump_parabola(
        vertical_position, max_altitude
    )
    #TODO: figure out how to not have to update these every single frame please
    sprite_node.material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_node.material.set_shader_parameter("ascending", is_ascending())
    sprite_node.material.set_shader_parameter("altitude_normal", altitude_normal)

    _update_sprite_scale(sprite_node, altitude_normal)
    PerspectiveTiltMask.update_cpu_side_sprite_data_ssbo_cache(
        sprite_texture_index,
        sprite_node.global_position,
        (sprite_node.texture.get_size() / 2.0) * sprite_node.scale,
        altitude_normal,
        1.0 if is_ascending() else 0.0
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


func _update_sprite_scale(sprite_node: Sprite2D, altitude_location: float) -> void:
    var scale_minimum: float = 1.0
    #var scale_multiplier: float = lerp(1.0, PARAMETERS.SPRITE_SCALE_AT_MAX_ALTITUDE, altitude_location)
    #TODO: below is an explicit lerp function
    var scale_multiplier: float = (
        scale_minimum
        + ((PARAMETERS.SPRITE_SCALE_AT_MAX_ALTITUDE - scale_minimum) * altitude_location)
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
        else SpacetimeContext.GRAVITY
    )
