extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

var mechanic_type: TYPE
var character_body: CapsuleDummy
var active_collision_shape: CollisionShape2D
var sprite_texture_index: int  #TODO: this is hacked, idk cant tell until having multiple sprites


func get_collision_shape() -> CollisionShape2D:
    if active_collision_shape == null:
        for child: Node in character_body.get_children():
            if child is CollisionShape2D:
                active_collision_shape = child
                break
    return active_collision_shape


func process_input(_delta: float) -> void:
    pass


func emit_mechanic_data(_frame_delta: float) -> void:
    pass


func process_collision_shape(_delta: float) -> void:
    pass
