extends Node
class_name MechanicController

signal strafe_left
signal strafe_right

enum STATE { DIVE, DIVE_ASCEND, JUMP, IDLE }
signal state_changed(new_state: MechanicController.STATE)
var current_state: MechanicController.STATE = MechanicController.STATE.DIVE
var queued_state: MechanicController.STATE = MechanicController.STATE.IDLE

var mechanic_scenes: Array[PackedScene] = [
    preload(ResourcePaths.STRAFE_MECHANIC),
    preload(ResourcePaths.JUMP_MECHANIC),
    preload(ResourcePaths.DIVE_MECHANIC),
]

var velocity: Vector2


func _ready() -> void:
    for mechanic_scene: PackedScene in mechanic_scenes:
        var mechanic: Node = mechanic_scene.instantiate()
        mechanic.velocity = velocity
        state_changed.connect(mechanic.on_state_changed)
        mechanic.state_completed.connect(_on_state_completed)
        if mechanic is Strafe:
            strafe_left.connect(mechanic.on_strafe_left)
            strafe_right.connect(mechanic.on_strafe_right)
        add_child(mechanic)

    state_changed.emit(current_state)


func _physics_process(_delta: float) -> void:
    self.velocity = Vector2.ZERO
    if Input.is_action_pressed("left"):
        strafe_left.emit()
    if Input.is_action_pressed("right"):
        strafe_right.emit()
    if Input.is_action_pressed("jump"):
        if current_state == MechanicController.STATE.DIVE:
            queued_state = MechanicController.STATE.JUMP
            _update_state(MechanicController.STATE.DIVE_ASCEND)
    #TODO: this feels hacked
    for mechanic: Node in get_children():
        self.velocity += mechanic.velocity


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
