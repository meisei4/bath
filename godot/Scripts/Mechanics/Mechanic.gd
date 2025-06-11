extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

var type: TYPE
var sprite_texture_index: int  #TODO: this is hacked, idk cant tell until having multiple sprites


func process_input(_delta: float) -> void:
    pass


func emit_mechanic_data(_frame_delta: float) -> void:
    pass
