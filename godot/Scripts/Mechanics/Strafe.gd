extends Node
class_name Strafe

signal state_completed(completed_state: MechanicController.STATE)

var mut_ref_velocity: MutRefVelocity
var strafe_data: StrafeData
var direction: int = 0

enum StrafePhase { LEFT, RIGHT, IDLE }


func _ready() -> void:
    if !strafe_data:
        strafe_data = StrafeData.new()
    set_physics_process(true)  # passive mechanic


func on_state_changed(state: MechanicController.STATE) -> void:
    pass


func _physics_process(delta: float) -> void:
    direction = 0
    if Input.is_action_pressed("left"):
        _strafe_left()
    if Input.is_action_pressed("right"):
        _strafe_right()

    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _apply_movement_input(time_scaled_delta)


func _apply_movement_input(delta: float) -> void:
    if direction != 0:
        mut_ref_velocity.val.x += strafe_data.ACCELERATION * delta * direction
        mut_ref_velocity.val.x = clamp(
            mut_ref_velocity.val.x, -strafe_data.MAX_SPEED, strafe_data.MAX_SPEED
        )
    else:
        if mut_ref_velocity.val.x > 0.0:
            mut_ref_velocity.val.x = max(
                0.0, mut_ref_velocity.val.x - strafe_data.DECELERATION * delta
            )
        elif mut_ref_velocity.val.x < 0.0:
            mut_ref_velocity.val.x = min(
                0.0, mut_ref_velocity.val.x + strafe_data.DECELERATION * delta
            )


func _strafe_left() -> void:
    direction = -1


func _strafe_right() -> void:
    direction = 1


func _handles_state(state: MechanicController.STATE) -> bool:
    return true
