extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

var type: TYPE

var delta_pixels: Vector2 = Vector2(0.0, 0.0)


func _ready() -> void:
    MechanicManager.state_changed.connect(_on_state_changed)


signal state_completed(finished_state: MechanicManager.STATE)


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
