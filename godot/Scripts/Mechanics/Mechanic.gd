extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

signal state_completed(completed_state: MechanicController.STATE)

var type: TYPE

var mechanic_controller: MechanicController


#TODO: this feels so decieving... i hate it
func _ready() -> void:
    mechanic_controller = get_parent()
    if mechanic_controller is MechanicController:
        mechanic_controller.state_changed.connect(_on_state_changed)


func _physics_process(_delta: float) -> void:
    pass


func _on_state_changed(state: MechanicController.STATE) -> void:
    pass



func emit_mechanic_data(_frame_delta: float) -> void:
    pass


func update_collision() -> void:
    pass


func handles_state(state: MechanicController.STATE) -> bool:
    return false
