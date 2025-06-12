extends Node
class_name MechanicController

signal left_lateral_movement
signal right_lateral_movement
signal jump_override
signal state_changed(new_state: MechanicController.STATE)

enum STATE { SWIM, SWIM_ASCEND, JUMP, NOTHING }

var mechanic_scenes: Array[PackedScene]

var current_state: MechanicController.STATE = MechanicController.STATE.SWIM
var queued_state: MechanicController.STATE = MechanicController.STATE.NOTHING

var controller_host: CharacterBody2D
var mechanics: Array[Mechanic]


func _ready() -> void:
    controller_host = get_parent() as CapsuleDummy
    _init_mechanics()
    jump_override.connect(_on_jump_override)
    state_changed.emit(current_state)


func handle_input() -> void:
    if Input.is_action_pressed("left"):
        left_lateral_movement.emit()
    if Input.is_action_pressed("right"):
        right_lateral_movement.emit()
    if Input.is_action_pressed("jump"):
        jump_override.emit()


func _on_jump_override() -> void:
    if current_state == MechanicController.STATE.SWIM:
        queued_state = MechanicController.STATE.JUMP
        _switch_state(MechanicController.STATE.SWIM_ASCEND)


func _on_state_completed(completed_state: MechanicController.STATE) -> void:
    if completed_state != current_state:
        return
    if queued_state != MechanicController.STATE.NOTHING and queued_state != current_state:
        var next: MechanicController.STATE = queued_state
        queued_state = MechanicController.STATE.NOTHING
        _switch_state(next)
    elif current_state == MechanicController.STATE.JUMP:
        _switch_state(MechanicController.STATE.SWIM)


func _switch_state(next_state: MechanicController.STATE) -> void:
    if current_state == next_state:
        return
    current_state = next_state
    state_changed.emit(next_state)

    if queued_state != MechanicController.STATE.NOTHING and queued_state != current_state:
        var pending_state: MechanicController.STATE = queued_state
        queued_state = MechanicController.STATE.NOTHING
        _switch_state(pending_state)


func _init_mechanics() -> void:
    for mechanic_scene: PackedScene in mechanic_scenes:
        var mechanic: Mechanic = mechanic_scene.instantiate()
        add_child(mechanic)
        mechanics.append(mechanic)
        mechanic.state_completed.connect(_on_state_completed)
