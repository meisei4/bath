extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

var type: TYPE

var delta_pixels: Vector2 = Vector2(0.0, 0.0)


func update_position_delta_pixels(_delta: float) -> void:
    pass


func emit_mechanic_data(_frame_delta: float) -> void:
    pass


func update_collision(collision_shape: CollisionShape2D) -> void:
    pass


func handles_state(state: MechanicManager.STATE) -> bool:
    return false
