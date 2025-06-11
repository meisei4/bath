extends Node
#class_name MechanicsManager

signal left_lateral_movement
signal right_lateral_movement
signal jump_override
signal character_body_registered(character_body: CharacterBody2D)
signal active_mechanic_changed(character_body: CharacterBody2D, next_mechanic: Mechanic)

enum State { SWIM, SWIM_ASCEND, JUMP }


class Controller:
    var character: CapsuleDummy
    var state: State = State.SWIM


var controller: Controller

var character_bodies: Array[CharacterBody2D]


func _ready() -> void:
    set_process(false)
    jump_override.connect(_on_jump_override)


func register_character_body(_character_body: CharacterBody2D) -> void:
    character_bodies.append(_character_body)
    character_body_registered.emit(_character_body)


func register_controller(body: CapsuleDummy) -> void:
    controller = Controller.new()
    controller.character = body
    for type: Mechanic.TYPE in controller.character.mechanics.keys():
        var mechanic: Mechanic = controller.character.mechanics[type]
        var is_passive_mechanic: bool = type == Mechanic.TYPE.LATERAL_MOVEMENT
        mechanic.set_process(is_passive_mechanic)
        mechanic.set_physics_process(is_passive_mechanic)

    _activate_mechanic(controller.character.mechanics[Mechanic.TYPE.SWIM])
    set_process(true)  #only run this entire manager when a controller is registered


func _process(delta: float) -> void:
    _handle_input()
    controller.character.mechanics[Mechanic.TYPE.LATERAL_MOVEMENT].emit_mechanic_data(delta)
    match controller.state:
        State.SWIM:
            controller.character.mechanics[Mechanic.TYPE.SWIM].emit_mechanic_data(delta)
        State.SWIM_ASCEND:
            _swim_ascend(delta)
        State.JUMP:
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
    for type: Mechanic.TYPE in controller.character.mechanics.keys():
        var mechanic: Mechanic = controller.character.mechanics[type]
        if type == Mechanic.TYPE.LATERAL_MOVEMENT:  #TODO: bad design
            continue
        var on: bool = mechanic == next_mechanic
        mechanic.set_process(on)
        mechanic.set_physics_process(on)

    active_mechanic_changed.emit(controller.character, next_mechanic)


func _switch_state(next_state: State) -> void:
    controller.state = next_state
    match next_state:
        State.SWIM, State.SWIM_ASCEND:
            _activate_mechanic(controller.character.mechanics[Mechanic.TYPE.SWIM])
        State.JUMP:
            _activate_mechanic(controller.character.mechanics[Mechanic.TYPE.JUMP])
            controller.character.mechanics[Mechanic.TYPE.JUMP]._on_jump()  #TODO: bad design


func _on_jump_override() -> void:  #TODO: bad naming
    if controller.state == State.SWIM:
        _switch_state(State.SWIM_ASCEND)


func _jump(delta: float) -> void:  #TODO: bad naming
    var jump: Jump = controller.character.mechanics[Mechanic.TYPE.JUMP] as Jump
    jump.emit_mechanic_data(delta)
    if jump._is_grounded():
        _switch_state(State.SWIM)


func _swim_ascend(delta: float) -> void:
    var swim: Swim = controller.character.mechanics[Mechanic.TYPE.SWIM] as Swim
    if swim.target_depth_position != Swim.LEVEL_DEPTH:
        swim.target_depth_position = Swim.LEVEL_DEPTH
        swim._set_phase(Swim.DivePhase.ASCENDING)

    swim.emit_mechanic_data(delta)
    if swim.current_depth_position >= Swim.LEVEL_DEPTH:
        _switch_state(State.JUMP)
