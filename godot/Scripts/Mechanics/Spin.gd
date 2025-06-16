extends Node
class_name Spin

signal animate_mechanic(mechanic_animation_data: MechanicAnimationData)

signal state_completed(completed_state: MechanicController.STATE)

var mechanic_animation_data: MechanicAnimationData
var mut_ref_velocity: MutRefVelocity

const SPIN_DURATION: float = 0.5  # seconds for a full 2π spin
var spin_speed: float = 1.0 / SPIN_DURATION
var spin_normal: float = 0.0


func _ready() -> void:
    mechanic_animation_data = MechanicAnimationData.new()
    set_physics_process(false)


func on_state_changed(state: MechanicController.STATE) -> void:
    if _handles_state(state):
        set_physics_process(true)
        _spin()


func _spin() -> void:
    spin_normal = 0.0


func _physics_process(delta: float) -> void:
    if spin_normal < 1.0:
        spin_normal += spin_speed * delta
        spin_normal = min(spin_normal, 1.0)  # Clamp to 1.0
        _emit_animation_data()

        if spin_normal >= 1.0:
            _complete_spin()


func _complete_spin() -> void:
    set_physics_process(false)
    state_completed.emit(MechanicController.STATE.SPIN)


func _emit_animation_data() -> void:
    mechanic_animation_data.vertical_normal = spin_normal
    animate_mechanic.emit(mechanic_animation_data)


func _handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.SPIN
