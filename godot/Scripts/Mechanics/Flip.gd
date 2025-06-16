extends Node
class_name Flip

signal animate_mechanic(mechanic_animation_data: MechanicAnimationData)

signal state_completed(completed_state: MechanicController.STATE)

var mechanic_animation_data: MechanicAnimationData
var mut_ref_velocity: MutRefVelocity

const FLIP_DURATION := 0.5  # seconds for a full 2π flip
var flip_speed := 1 / FLIP_DURATION
var flip_normal := 0.0


func _ready() -> void:
    mechanic_animation_data = MechanicAnimationData.new()
    set_physics_process(false)


func on_state_changed(state: MechanicController.STATE) -> void:
    if _handles_state(state):
        set_physics_process(true)
        _flip()


func _flip() -> void:
    flip_normal = 0.0

func _physics_process(delta: float) -> void:
    if flip_normal < 1.0:
        flip_normal += flip_speed * delta
        flip_normal = min(flip_normal, 1.0)  # Clamp to 1.0
        _emit_animation_data()

        if flip_normal >= 1.0:
            _complete_flip()

func _complete_flip() -> void:
    set_physics_process(false)
    state_completed.emit(MechanicController.STATE.FLIP)

func _emit_animation_data() -> void:
    mechanic_animation_data.vertical_normal = flip_normal
    animate_mechanic.emit(mechanic_animation_data)

func _handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.FLIP
