extends Node
class_name CollisionController

var mechanics: Array[Node]
var jump: Jump
var dive: Dive


func _ready() -> void:
    for mechanic: Node in mechanics:
        if mechanic is Jump:
            jump = mechanic
        if mechanic is Dive:
            dive = mechanic


func collision_shape_disabled() -> bool:
    return (
        jump.current_phase != Jump.JumpPhase.GROUNDED or dive.current_phase != Dive.DivePhase.LEVEL
    )
