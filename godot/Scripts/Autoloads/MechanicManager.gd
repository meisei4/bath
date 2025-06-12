extends Node
#class_name MechanicManager

signal left_lateral_movement
signal right_lateral_movement
signal jump_override
signal character_body_registered(character_body: CharacterBody2D)
signal active_mechanic_changed(character_body: CharacterBody2D, next_mechanic: Mechanic)

enum STATE { SWIM, SWIM_ASCEND, JUMP }


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
        var is_passive_mechanic: bool = mechanic.type == Mechanic.TYPE.LATERAL_MOVEMENT
        mechanic.set_process(is_passive_mechanic)
        mechanic.set_physics_process(is_passive_mechanic)

    var swim_mechanic: Mechanic = null
    for mechanic: Mechanic in controller.character.mechanics:
        if mechanic.type == Mechanic.TYPE.SWIM:
            swim_mechanic = mechanic
            break

    _activate_mechanic(swim_mechanic)
    set_process(true)


func _process(delta: float) -> void:
    _handle_input()
    for mechanic: Mechanic in controller.character.mechanics:
        if mechanic.type == Mechanic.TYPE.LATERAL_MOVEMENT:
            mechanic.emit_mechanic_data(delta)
            break

    match controller.state:
        STATE.SWIM:
            for mechanic: Mechanic in controller.character.mechanics:
                if mechanic.type == Mechanic.TYPE.SWIM:
                    mechanic.emit_mechanic_data(delta)
                    break
        STATE.SWIM_ASCEND:
            _swim_ascend(delta)
        STATE.JUMP:
            _jump(delta)


func _handle_input() -> void:
    if Input.is_action_pressed("left"):
        left_lateral_movement.emit()
    if Input.is_action_pressed("right"):
        right_lateral_movement.emit()


func _unhandled_input(event: InputEvent) -> void:
    if event is InputEventKey and event.pressed and event.keycode == Key.KEY_SPACE:
        jump_override.emit()


func _activate_mechanic(next_mechanic: Mechanic) -> void:
    for mechanic: Mechanic in controller.character.mechanics:
        if mechanic.type == Mechanic.TYPE.LATERAL_MOVEMENT:
            continue
        var on: bool = mechanic == next_mechanic
        mechanic.set_process(on)
        mechanic.set_physics_process(on)

    active_mechanic_changed.emit(controller.character, next_mechanic)


func _switch_state(next_state: STATE) -> void:
    controller.state = next_state
    match next_state:
        STATE.SWIM, STATE.SWIM_ASCEND:
            for mechanic: Mechanic in controller.character.mechanics:
                if mechanic.type == Mechanic.TYPE.SWIM:
                    _activate_mechanic(mechanic)
                    break
        STATE.JUMP:
            for mechanic: Mechanic in controller.character.mechanics:
                if mechanic.type == Mechanic.TYPE.JUMP:
                    _activate_mechanic(mechanic)
                    mechanic._on_jump()  #TODO: bad design
                    break


func _on_jump_override() -> void:  #TODO: bad naming
    if controller.state == STATE.SWIM:
        _switch_state(STATE.SWIM_ASCEND)


func _jump(delta: float) -> void:  #TODO: bad naming
    for mechanic: Mechanic in controller.character.mechanics:
        if mechanic.type == Mechanic.TYPE.JUMP:
            var jump: Jump = mechanic as Jump
            jump.emit_mechanic_data(delta)
            if jump._is_grounded():
                _switch_state(STATE.SWIM)
            break


func _swim_ascend(delta: float) -> void:
    for mechanic: Mechanic in controller.character.mechanics:
        if mechanic.type == Mechanic.TYPE.SWIM:
            var swim: Swim = mechanic as Swim
            if swim.target_depth_position != Swim.LEVEL_DEPTH:
                swim.target_depth_position = Swim.LEVEL_DEPTH
                swim._set_phase(Swim.DivePhase.ASCENDING)

            swim.emit_mechanic_data(delta)
            if swim.current_depth_position >= Swim.LEVEL_DEPTH:
                _switch_state(STATE.JUMP)
            break
