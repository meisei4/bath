extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

signal state_completed(finished_state: MechanicManager.STATE)

var type: TYPE

var delta_pixels: Vector2 = Vector2(0.0, 0.0)

var mechanic_controller: MechanicController


func _ready() -> void:
    mechanic_controller = get_parent()  # our direct parent
    if mechanic_controller is MechanicController:
        mechanic_controller.state_changed.connect(_on_state_changed)


func _ready1() -> void:
    MechanicManager.state_changed.connect(_on_state_changed)


func _on_state_changed(state: MechanicManager.STATE) -> void:
    var active: bool = handles_state(state)
    set_process(active)
    set_physics_process(active)


func update_position_delta_pixels(_delta: float) -> void:
    pass


func emit_mechanic_data(_frame_delta: float) -> void:
    pass


func update_collision(collision_shape: CollisionShape2D) -> void:
    pass


func handles_state(state: MechanicManager.STATE) -> bool:
    return false
