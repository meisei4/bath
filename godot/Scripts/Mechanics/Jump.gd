extends Node
class_name Jump

signal animate_mechanic(mechanic_animation_data: MechanicAnimationData)

signal state_completed(completed_state: MechanicController.STATE)

var jump_data: JumpData
var mechanic_animation_data: MechanicAnimationData
var mut_ref_velocity: MutRefVelocity
var current_vertical_velocity: float
var current_altitude_position: float

enum JumpPhase { GROUNDED, ASCENDING, DESCENDING }
var current_phase: JumpPhase = JumpPhase.GROUNDED


func _ready() -> void:
    mechanic_animation_data = MechanicAnimationData.new()
    if !jump_data:
        jump_data = JumpData.new()
    set_physics_process(false)


func on_state_changed(state: MechanicController.STATE) -> void:
    if _handles_state(state):
        set_physics_process(true)
        _jump()


func _jump() -> void:
    if current_phase == JumpPhase.GROUNDED:
        current_altitude_position = jump_data.INITIAL_VERTICAL_POSITION
        current_vertical_velocity = jump_data.INITIAL_JUMP_VELOCITY
        _set_phase(JumpPhase.ASCENDING)


func _physics_process(delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _apply_gravity(time_scaled_delta)
    _update_altitude(time_scaled_delta)
    mut_ref_velocity.val.y = -jump_data.FORWARD_VELOCITY
    _emit_animation_data()
    if current_phase == JumpPhase.DESCENDING and current_altitude_position <= 0.0:
        _handle_landing()


func _apply_gravity(delta: float) -> void:
    var gravity: float = _get_effective_gravity()
    current_vertical_velocity -= gravity * delta


func _update_altitude(delta: float) -> void:
    current_altitude_position += current_vertical_velocity * delta
    if current_vertical_velocity > 0.0:
        _set_phase(JumpPhase.ASCENDING)
    elif current_vertical_velocity < 0.0 and current_altitude_position > 0.0:
        _set_phase(JumpPhase.DESCENDING)


func _handle_landing() -> void:
    set_physics_process(false)
    current_altitude_position = 0.0
    current_vertical_velocity = 0.0
    mut_ref_velocity.val.y = 0.0
    _set_phase(JumpPhase.GROUNDED)
    state_completed.emit(MechanicController.STATE.JUMP)


func _set_phase(new_phase: JumpPhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase


func _get_effective_gravity() -> float:
    return (
        jump_data.OVERRIDE_GRAVITY if jump_data.OVERRIDE_GRAVITY > 0.0 else SpacetimeManager.GRAVITY
    )


func _emit_animation_data() -> void:
    mechanic_animation_data.current_vertical_position = current_altitude_position
    mechanic_animation_data.vertical_normal = _compute_altitude_normal()
    mechanic_animation_data.ascending = current_phase == JumpPhase.ASCENDING
    animate_mechanic.emit(mechanic_animation_data)


func _compute_altitude_normal() -> float:
    var altitude_normal: float = current_altitude_position / _max_altitude()
    return clamp(altitude_normal, 0.0, 1.0)


func _max_altitude() -> float:
    var velocity_squared: float = pow(jump_data.INITIAL_JUMP_VELOCITY, 2)
    var denominator: float = 2.0 * _get_effective_gravity()
    return velocity_squared / denominator


func _handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.JUMP
