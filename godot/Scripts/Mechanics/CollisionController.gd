extends Node
class_name CollisionController

@export var mechanics: Array[Node]
@export var jump: Jump
@export var dive: Dive


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
