extends Node
class_name MechanicController

signal left_lateral_movement
signal right_lateral_movement
signal jump_override
signal state_changed(new_state: MechanicManager.STATE)

@export var mechanic_scenes: Array[PackedScene]

var current_state: MechanicManager.STATE = MechanicManager.STATE.SWIM
var queued_state: MechanicManager.STATE = MechanicManager.STATE.NOTHING

var controller_host: CharacterBody2D
var mechanics: Array[Mechanic]


func _ready() -> void:
    controller_host = get_parent() as CapsuleDummy
    _init_mechanics()
    _connect_passive_flags()
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
    if current_state == MechanicManager.STATE.SWIM:
        queued_state = MechanicManager.STATE.JUMP
        _switch_state(MechanicManager.STATE.SWIM_ASCEND)


func _on_state_completed(completed_state: MechanicManager.STATE) -> void:
    if completed_state != current_state:
        return
    if queued_state != MechanicManager.STATE.NOTHING and queued_state != current_state:
        var next: MechanicManager.STATE = queued_state
        queued_state = MechanicManager.STATE.NOTHING
        _switch_state(next)
    elif current_state == MechanicManager.STATE.JUMP:
        _switch_state(MechanicManager.STATE.SWIM)


func _switch_state(next_state: MechanicManager.STATE) -> void:
    if current_state == next_state:
        return
    current_state = next_state
    state_changed.emit(next_state)

    if queued_state != MechanicManager.STATE.NOTHING and queued_state != current_state:
        var pending_state: MechanicManager.STATE = queued_state
        queued_state = MechanicManager.STATE.NOTHING
        _switch_state(pending_state)


func _init_mechanics() -> void:
    for scene: PackedScene in mechanic_scenes:
        var mechanic: Mechanic = scene.instantiate()
        add_child(mechanic)
        mechanics.append(mechanic)
        mechanic.state_completed.connect(_on_state_completed)
        if mechanic.type == Mechanic.TYPE.LATERAL_MOVEMENT:
            mechanic.set_process(true)
            mechanic.set_physics_process(true)


func _connect_passive_flags() -> void:
    for mechanic: Mechanic in mechanics:
        if mechanic.type != Mechanic.TYPE.LATERAL_MOVEMENT:
            var active: bool = mechanic.handles_state(current_state)
            mechanic.set_process(active)
            mechanic.set_physics_process(active)
