extends Node
class_name Cruising

signal state_completed(completed_state: MechanicController.STATE)

var mut_ref_velocity: MutRefVelocity
@export var cruising_data: CruisingData
var direction: int = 0


func _ready() -> void:
    if !cruising_data:
        cruising_data = CruisingData.new()
    set_physics_process(true)  # passive mechanic


func on_state_changed(state: MechanicController.STATE) -> void:
    pass


func _physics_process(delta: float) -> void:
    direction = 0
    if Input.is_action_pressed("up"):
        _forward()
    if Input.is_action_pressed("down"):
        _backward()

    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _apply_movement_input(time_scaled_delta)


func _apply_movement_input(delta: float) -> void:
    if direction != 0:
        mut_ref_velocity.val.y += cruising_data.ACCELERATION * delta * direction
        mut_ref_velocity.val.y = clamp(
            mut_ref_velocity.val.y, -cruising_data.MAX_SPEED, cruising_data.MAX_SPEED
        )
    else:
        if mut_ref_velocity.val.y > 0.0:
            mut_ref_velocity.val.y = max(
                0.0, mut_ref_velocity.val.y - cruising_data.DECELERATION * delta
            )
        elif mut_ref_velocity.val.y < 0.0:
            mut_ref_velocity.val.y = min(
                0.0, mut_ref_velocity.val.y + cruising_data.DECELERATION * delta
            )


func _forward() -> void:
    direction = -1


func _backward() -> void:
    direction = 1


func _handles_state(state: MechanicController.STATE) -> bool:
    return true
