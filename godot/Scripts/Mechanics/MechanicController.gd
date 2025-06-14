extends Node
class_name MechanicController

signal left_lateral_movement
signal right_lateral_movement
signal jump_override

enum STATE { DIVE, DIVE_ASCEND, JUMP, IDLE }
signal state_changed(new_state: MechanicController.STATE)
var current_state: MechanicController.STATE = MechanicController.STATE.DIVE
var queued_state: MechanicController.STATE = MechanicController.STATE.IDLE

var mechanic_scenes: Array[PackedScene] = [
    preload(ResourcePaths.STRAFE_MECHANIC),
    preload(ResourcePaths.JUMP_MECHANIC),
    preload(ResourcePaths.DIVE_MECHANIC),
]

var controller_host: CharacterBody2D


func _ready() -> void:
    if !controller_host:
        print("no controller host, bad")
        return

    for mechanic_scene: PackedScene in mechanic_scenes:
        var mechanic: Node = mechanic_scene.instantiate()
        mechanic.mechanic_controller = self
        add_child(mechanic)
        mechanic.state_completed.connect(_on_state_completed)

    jump_override.connect(_on_jump_override)
    state_changed.emit(current_state)


func _physics_process(_delta: float) -> void:
    if Input.is_action_pressed("left"):
        left_lateral_movement.emit()
    if Input.is_action_pressed("right"):
        right_lateral_movement.emit()
    if Input.is_action_pressed("jump"):
        jump_override.emit()


func _on_jump_override() -> void:
    if current_state == MechanicController.STATE.DIVE:
        queued_state = MechanicController.STATE.JUMP
        _update_state(MechanicController.STATE.DIVE_ASCEND)


func _update_state(next_state: MechanicController.STATE) -> void:
    if current_state != next_state:
        current_state = next_state
        state_changed.emit(next_state)


func _on_state_completed(completed_state: MechanicController.STATE) -> void:
    if completed_state != current_state:
        print("you are completing some non-current state, thats bad")
        return
    if queued_state != MechanicController.STATE.IDLE and queued_state != current_state:
        var next: MechanicController.STATE = queued_state
        queued_state = MechanicController.STATE.IDLE
        _update_state(next)
    elif current_state == MechanicController.STATE.JUMP:
        _update_state(MechanicController.STATE.DIVE)
