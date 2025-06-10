extends Node
#class_name MechanicManager

signal left_lateral_movement
signal right_lateral_movement
signal jump_override

enum State { SWIM, SWIM_ASCEND, JUMP }


class Controller:
    var character: CapsuleDummy
    var state: State = State.SWIM


var controller: Controller
var active_shader: Shader


func register_controller(body: CapsuleDummy) -> void:
    controller = Controller.new()
    controller.character = body
    for mechanic_type: Mechanic.TYPE in controller.character.mechanics.keys():
        var mechanic: Mechanic = controller.character.mechanics[mechanic_type]
        var is_passive_mechanic: bool = mechanic_type == Mechanic.TYPE.LATERAL_MOVEMENT
        mechanic.set_process(is_passive_mechanic)
        mechanic.set_physics_process(is_passive_mechanic)

    _activate_mechanic(controller.character.mechanics[Mechanic.TYPE.SWIM])
    set_process(true)  #only run this entire manager when a controller is registered


func _ready() -> void:
    set_process(false)
    jump_override.connect(_on_jump_override)


func _process(delta: float) -> void:
    _handle_input()
    _run_mechanic(controller.character.mechanics[Mechanic.TYPE.LATERAL_MOVEMENT], delta)
    match controller.state:
        State.SWIM:
            _run_mechanic(controller.character.mechanics[Mechanic.TYPE.SWIM], delta)
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
    for mechanic_type: Mechanic.TYPE in controller.character.mechanics.keys():
        var mechanic: Mechanic = controller.character.mechanics[mechanic_type]
        if mechanic_type == Mechanic.TYPE.LATERAL_MOVEMENT:  #TODO: bad design
            continue
        var on: bool = mechanic == next_mechanic
        mechanic.set_process(on)
        mechanic.set_physics_process(on)

    var shader: Shader = next_mechanic.mechanic_shader
    if shader == null or shader == active_shader:
        return

    var sprite: Sprite2D = next_mechanic.get_sprite()
    if sprite:
        if sprite.material:
            sprite.material.shader = shader
            active_shader = shader
        else:
            sprite.material = ShaderMaterial.new()


func _run_mechanic(mechanic: Mechanic, delta: float) -> void:
    mechanic.process_input(delta)
    mechanic.process_collision_shape(delta)
    mechanic.process_visual_illusion(delta)


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
    _run_mechanic(jump, delta)
    if jump._is_grounded():
        _switch_state(State.SWIM)


func _swim_ascend(delta: float) -> void:
    var swim: Swim = controller.character.mechanics[Mechanic.TYPE.SWIM] as Swim
    if swim.target_depth_position != Swim.LEVEL_DEPTH:
        swim.target_depth_position = Swim.LEVEL_DEPTH
        swim._set_phase(Swim.DivePhase.ASCENDING)

    _run_mechanic(swim, delta)
    if swim.current_depth_position >= Swim.LEVEL_DEPTH:
        _switch_state(State.JUMP)
