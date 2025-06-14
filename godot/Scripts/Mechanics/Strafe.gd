extends Node
class_name Strafe

signal state_completed(completed_state: MechanicController.STATE)

var mechanic_controller: MechanicController

const MAX_SPEED: float = 60.0
const ACCELERATION: float = 4000.0
const DECELERATION: float = 2000.0

var movement_input: int = 0
var stretch_timer: float = 0.0


func _ready() -> void:
    if !mechanic_controller:
        print("no mechanic controller, bad")
        return

    mechanic_controller.left_lateral_movement.connect(_on_move_left_triggered)
    mechanic_controller.right_lateral_movement.connect(_on_move_right_triggered)
    set_physics_process(true)  # passive mechanic


func _physics_process(delta: float) -> void:
    var time: float = SpacetimeManager.apply_time_scale(delta)
    mechanic_controller.controller_host.velocity.x = SpacetimeManager.apply_universal_drag(
        mechanic_controller.controller_host.velocity.x, time
    )
    _apply_movement_input(time)
    _apply_cosmic_friction(time)
    movement_input = 0


func _apply_movement_input(time: float) -> void:
    if movement_input != 0:
        mechanic_controller.controller_host.velocity.x += (
            ACCELERATION * time * float(movement_input)
        )
        mechanic_controller.controller_host.velocity.x = clamp(
            mechanic_controller.controller_host.velocity.x, -MAX_SPEED, MAX_SPEED
        )
    else:
        if mechanic_controller.controller_host.velocity.x > 0.0:
            mechanic_controller.controller_host.velocity.x = max(
                0.0, mechanic_controller.controller_host.velocity.x - DECELERATION * time
            )
        elif mechanic_controller.controller_host.velocity.x < 0.0:
            mechanic_controller.controller_host.velocity.x = min(
                0.0, mechanic_controller.controller_host.velocity.x + DECELERATION * time
            )


func _apply_cosmic_friction(time: float) -> void:
    var friction_amount: float = SpacetimeManager.COSMIC_FRICTION * time
    if mechanic_controller.controller_host.velocity.x > 0.0:
        mechanic_controller.controller_host.velocity.x = max(
            0.0, mechanic_controller.controller_host.velocity.x - friction_amount
        )
    elif mechanic_controller.controller_host.velocity.x < 0.0:
        mechanic_controller.controller_host.velocity.x = min(
            0.0, mechanic_controller.controller_host.velocity.x + friction_amount
        )


func _on_move_left_triggered() -> void:
    movement_input = -1


func _on_move_right_triggered() -> void:
    movement_input = 1
