extends Node
class_name MechanicController

enum STATE { DIVE, DIVE_ASCEND, JUMP, IDLE, SPIN }
signal state_changed(new_state: MechanicController.STATE)
@export var current_state: MechanicController.STATE = MechanicController.STATE.DIVE
var queued_state: MechanicController.STATE = MechanicController.STATE.IDLE

@export var mechanic_scenes: Array[PackedScene] = [
    preload(ResourcePaths.STRAFE_MECHANIC),
    preload(ResourcePaths.CRUISING_MECHANIC),
    preload(ResourcePaths.JUMP_MECHANIC),
    preload(ResourcePaths.DIVE_MECHANIC),
    preload(ResourcePaths.SPIN_MECHANIC),
]

@export var mut_ref_velocity: MutRefVelocity


func _ready() -> void:
    for mechanic_scene: PackedScene in mechanic_scenes:
        var mechanic: Node = mechanic_scene.instantiate()
        mechanic.mut_ref_velocity = mut_ref_velocity
        state_changed.connect(mechanic.on_state_changed)
        mechanic.state_completed.connect(_on_state_completed)
        add_child(mechanic)

    state_changed.emit(current_state)


func _physics_process(_delta: float) -> void:
    if Input.is_action_pressed("jump"):
        if current_state == MechanicController.STATE.DIVE:
            queued_state = MechanicController.STATE.JUMP
            _update_state(MechanicController.STATE.DIVE_ASCEND)
    if Input.is_action_pressed("F"):
        if current_state == MechanicController.STATE.JUMP:
            #queued_state = MechanicController.STATE.SPIN
            _update_state(MechanicController.STATE.SPIN)


func _update_state(next_state: MechanicController.STATE) -> void:
    if current_state != next_state:
        current_state = next_state
        state_changed.emit(next_state)


func _on_state_completed(completed_state: MechanicController.STATE) -> void:
    if completed_state != current_state:
        print("you are completing some non-current state, thats bad")
        return
    #TODO: I dont like this boolean spaghetti
    if queued_state != current_state and queued_state != MechanicController.STATE.IDLE:
        var next_state: MechanicController.STATE = queued_state
        queued_state = MechanicController.STATE.IDLE
        _update_state(next_state)
    elif current_state == MechanicController.STATE.JUMP:
        _update_state(MechanicController.STATE.DIVE)
    if current_state == MechanicController.STATE.SPIN:
        _update_state(MechanicController.STATE.JUMP)
