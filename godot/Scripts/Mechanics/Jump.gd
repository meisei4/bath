extends Mechanic
class_name Jump

signal animate_jump(vertical_position: float, altitude_normal: float, ascending: bool)

@export var PARAMETERS: JumpData

enum JumpPhase { GROUNDED, ASCENDING, DESCENDING }
var current_phase: JumpPhase = JumpPhase.GROUNDED
var vertical_speed: float = 0.0
var vertical_position: float = 0.0


func _ready() -> void:
    type = Mechanic.TYPE.JUMP
    if PARAMETERS == null:
        PARAMETERS = JumpData.new()


func update_position_delta_pixels(frame_delta: float) -> void:
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
    delta_pixels = Vector2(
        0.0, -1.0 * SpacetimeManager.to_physical_space(forward_movement_world_units)
    )


func emit_mechanic_data(_frame_delta: float) -> void:
    var max_altitude: float = _max_altitude()
    var altitude_normal: float = _compute_altitude_normal_in_jump_parabola(
        vertical_position, max_altitude
    )
    animate_jump.emit(vertical_position, altitude_normal, is_ascending())


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
    delta_pixels = Vector2(0.0, 0.0)
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


func update_collision(collision_shape: CollisionShape2D) -> void:
    if _is_grounded():
        collision_shape.disabled = false  #TODO: lmao double negatives
    else:
        collision_shape.disabled = true


func handles_state(state: MechanicManager.STATE) -> bool:
    return state == MechanicManager.STATE.JUMP
