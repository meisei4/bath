extends Node
class_name Strafe

signal state_completed(completed_state: MechanicController.STATE)

var velocity: Vector2
var strafe_data: StrafeData
var movement_input: int = 0

enum StrafePhase { LEFT, RIGHT, IDLE }


func _ready() -> void:
    if !strafe_data:
        strafe_data = StrafeData.new()
    set_physics_process(true)  # passive mechanic


func on_state_changed(state: MechanicController.STATE) -> void:
    pass


func _physics_process(delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _apply_movement_input(time_scaled_delta)
    movement_input = 0


func _apply_movement_input(delta: float) -> void:
    if movement_input != 0:
        velocity.x += strafe_data.ACCELERATION * delta * movement_input
        velocity.x = clamp(velocity.x, -strafe_data.MAX_SPEED, strafe_data.MAX_SPEED)
    else:
        if velocity.x > 0.0:
            velocity.x = max(0.0, velocity.x - strafe_data.DECELERATION * delta)
        elif velocity.x < 0.0:
            velocity.x = min(0.0, velocity.x + strafe_data.DECELERATION * delta)


func on_strafe_left() -> void:
    movement_input = -1


func on_strafe_right() -> void:
    movement_input = 1


func _handles_state(state: MechanicController.STATE) -> bool:
    return true
