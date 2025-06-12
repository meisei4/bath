extends Node
#class_name MechanicManager

signal left_lateral_movement
signal right_lateral_movement
signal jump_override
signal character_body_registered(character_body: CharacterBody2D)
signal active_mechanic_changed(character_body: CharacterBody2D, next_mechanic: Mechanic)
signal state_changed(new_state: STATE)

enum STATE { SWIM, SWIM_ASCEND, JUMP, NOTHING }

var queued_state: STATE = STATE.NOTHING


class Controller:
    var character: CapsuleDummy
    var state: STATE = STATE.SWIM


var controller: Controller


func _ready() -> void:
    set_process(false)
    jump_override.connect(_on_jump_override)


func register_character_body(_character_body: CharacterBody2D) -> void:
    character_body_registered.emit(_character_body)


func register_controller(body: CapsuleDummy) -> void:
    controller = Controller.new()
    controller.character = body
    for mechanic: Mechanic in controller.character.mechanics:
        mechanic.state_completed.connect(_on_state_completed)
        var is_passive_mechanic: bool = mechanic.type == Mechanic.TYPE.LATERAL_MOVEMENT
        mechanic.set_process(is_passive_mechanic)
        mechanic.set_physics_process(is_passive_mechanic)

    var swim_mechanic: Mechanic = null
    for mechanic: Mechanic in controller.character.mechanics:
        if mechanic.type == Mechanic.TYPE.SWIM:
            swim_mechanic = mechanic
            break
    state_changed.emit(controller.state)  # controller.state is SWIM see default at the top
    set_process(true)


func _process(delta: float) -> void:
    _handle_input()


func _handle_input() -> void:
    if Input.is_action_pressed("left"):
        left_lateral_movement.emit()
    if Input.is_action_pressed("right"):
        right_lateral_movement.emit()


func _unhandled_input(event: InputEvent) -> void:
    if event is InputEventKey and event.pressed and event.keycode == Key.KEY_SPACE:
        jump_override.emit()


func _on_state_completed(done_state: STATE) -> void:
    if done_state != controller.state:
        return

    if queued_state != STATE.NOTHING and queued_state != controller.state:
        var next: STATE = queued_state
        queued_state = STATE.NOTHING
        _switch_state(next)
    else:
        if controller.state == STATE.JUMP:
            _switch_state(STATE.SWIM)


func _switch_state(next_state: STATE) -> void:
    if controller.state == next_state:
        return

    controller.state = next_state
    state_changed.emit(next_state)
    if queued_state != STATE.NOTHING and queued_state != controller.state:
        var pending: STATE = queued_state
        queued_state = STATE.NOTHING
        _switch_state(pending)


func _on_jump_override() -> void:
    if controller.state == STATE.SWIM:
        queued_state = STATE.JUMP
        _switch_state(STATE.SWIM_ASCEND)
