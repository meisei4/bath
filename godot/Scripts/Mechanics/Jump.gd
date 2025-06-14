extends Node
class_name Jump

signal animate_jump(
    vertical_position: float, altitude_normal: float, ascending: bool, sprite: Sprite2D
)

signal state_completed(completed_state: MechanicController.STATE)

var mechanic_controller: MechanicController

var jump_data: JumpData

enum JumpPhase { GROUNDED, ASCENDING, DESCENDING }
var current_phase: JumpPhase = JumpPhase.GROUNDED
var vertical_velocity: float = 0.0
var vertical_position: float = 0.0

var animation: JumpAnimation


func _ready() -> void:
    if !mechanic_controller:
        print("no mechanic controller, bad")
        return

    mechanic_controller.state_changed.connect(_on_state_changed)
    animation = JumpAnimation.new()
    if !jump_data:
        jump_data = JumpData.new()
    set_physics_process(false)


func _on_state_changed(state: MechanicController.STATE) -> void:
    if handles_state(state):
        set_physics_process(true)
        animate_jump.connect(animation.process_animation)
        _jump()


func _jump() -> void:
    if !_is_airborne():
        vertical_velocity = jump_data.INITIAL_JUMP_VELOCITY
        _set_phase(JumpPhase.ASCENDING)


func _physics_process(delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _apply_gravity_and_drag(time_scaled_delta)
    _update_altitude(time_scaled_delta)
    if _should_land():
        _handle_landing()
    if _is_airborne() and jump_data.FORWARD_SPEED != 0.0:
        var scale: float = time_scaled_delta / delta
        mechanic_controller.controller_host.velocity.y = -SpacetimeManager.to_physical_space(
            jump_data.FORWARD_SPEED * scale
        )
    _update_collision()
    _emit_animation_data(delta)


func _apply_gravity_and_drag(time_scaled_delta: float) -> void:
    if _is_airborne():
        var gravity: float = _get_effective_gravity()
        vertical_velocity -= gravity * time_scaled_delta
        vertical_velocity = SpacetimeManager.apply_universal_drag(
            vertical_velocity, time_scaled_delta
        )


func _update_altitude(time_scaled_delta: float) -> void:
    vertical_position += vertical_velocity * time_scaled_delta
    if vertical_velocity > 0.0:
        _set_phase(JumpPhase.ASCENDING)
    elif vertical_velocity < 0.0 and vertical_position > 0.0:
        _set_phase(JumpPhase.DESCENDING)


func _handle_landing() -> void:
    set_physics_process(false)
    vertical_position = 0.0
    vertical_velocity = 0.0
    mechanic_controller.controller_host.velocity.y = 0.0
    _set_phase(JumpPhase.GROUNDED)
    animate_jump.disconnect(animation.process_animation)
    state_completed.emit(MechanicController.STATE.JUMP)


func _max_altitude() -> float:
    if _get_effective_gravity() > 0.0:
        var squared_initial_velocity: float = (
            jump_data.INITIAL_JUMP_VELOCITY * jump_data.INITIAL_JUMP_VELOCITY
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


func _set_phase(new_phase: JumpPhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase


func _get_effective_gravity() -> float:
    return (
        jump_data.OVERRIDE_GRAVITY if jump_data.OVERRIDE_GRAVITY > 0.0 else SpacetimeManager.GRAVITY
    )


func _is_grounded() -> bool:
    return current_phase == JumpPhase.GROUNDED


func _is_ascending() -> bool:
    return current_phase == JumpPhase.ASCENDING


func _is_descending() -> bool:
    return current_phase == JumpPhase.DESCENDING


func _is_airborne() -> bool:
    return current_phase != JumpPhase.GROUNDED


func _should_land() -> bool:
    return _is_descending() and vertical_position <= 0.0


func _update_collision() -> void:
    if _is_grounded():
        mechanic_controller.controller_host.collision_shape.disabled = false  #TODO: lmao double negatives
    else:
        mechanic_controller.controller_host.collision_shape.disabled = true


func _emit_animation_data(_frame_delta: float) -> void:
    var max_altitude: float = _max_altitude()
    var altitude_normal: float = _compute_altitude_normal_in_jump_parabola(
        vertical_position, max_altitude
    )
    animate_jump.emit(
        vertical_position,
        altitude_normal,
        _is_ascending(),
        mechanic_controller.controller_host.sprite
    )
    AnimationManager.update_perspective_tilt_mask(
        mechanic_controller.controller_host.sprite.texture,
        mechanic_controller.controller_host,
        mechanic_controller.controller_host.sprite.global_position,
        mechanic_controller.controller_host.sprite.scale,
        altitude_normal,
        _is_ascending()
    )


func handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.JUMP
