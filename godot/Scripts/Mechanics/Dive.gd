extends Node
class_name Dive

signal animate_mechanic(mechanic_animation_data: MechanicAnimationData)

signal state_completed(completed_state: MechanicController.STATE)

var dive_data: DiveData
var mechanic_animation_data: MechanicAnimationData
var mut_ref_velocity: MutRefVelocity
var in_queued_ascend: bool
var current_depth_position: float
var target_depth_position: float

enum DivePhase { LEVEL, ASCENDING, DIVING }
var current_phase: DivePhase = DivePhase.LEVEL

var debug_autoswim: bool = false #true
const _DEBUG_PERIOD: float = 3.0
var _debug_clock: float = 0.0


func _ready() -> void:
    if !dive_data:
        dive_data = DiveData.new()
    mechanic_animation_data = MechanicAnimationData.new()
    in_queued_ascend = false
    current_depth_position = dive_data.LEVEL_DEPTH
    target_depth_position = dive_data.LEVEL_DEPTH
    set_physics_process(false)


func on_state_changed(state: MechanicController.STATE) -> void:
    if _handles_state(state):
        set_physics_process(true)

    in_queued_ascend = (state == MechanicController.STATE.DIVE_ASCEND)
    if in_queued_ascend:
        target_depth_position = dive_data.LEVEL_DEPTH
        _set_phase(DivePhase.ASCENDING)


func _physics_process(delta: float) -> void:
    if debug_autoswim and !in_queued_ascend:
        _debug_clock += delta
        if _debug_clock >= _DEBUG_PERIOD:
            _debug_clock -= _DEBUG_PERIOD
            if target_depth_position == dive_data.LEVEL_DEPTH:
                target_depth_position = dive_data.MAX_DIVE_DEPTH
            else:
                target_depth_position = dive_data.LEVEL_DEPTH

    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _update_depth(time_scaled_delta)
    _emit_animation_data(time_scaled_delta)


func _update_depth(delta: float) -> void:
    var step: float = dive_data.DEPTH_SPEED * delta
    current_depth_position = move_toward(current_depth_position, target_depth_position, step)

    if abs(current_depth_position - dive_data.LEVEL_DEPTH) < dive_data.THRESHOLD:
        _set_phase(DivePhase.LEVEL)
        if in_queued_ascend:
            in_queued_ascend = false
            set_physics_process(false)
            state_completed.emit(MechanicController.STATE.DIVE_ASCEND)

        return
    if target_depth_position == dive_data.LEVEL_DEPTH:
        _set_phase(DivePhase.ASCENDING)
    else:
        _set_phase(DivePhase.DIVING)


func _set_phase(new_phase: DivePhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase


func _emit_animation_data(_frame_delta: float) -> void:
    mechanic_animation_data.current_vertical_position = current_depth_position
    mechanic_animation_data.vertical_normal = _compute_depth_normal()
    mechanic_animation_data.ascending = current_phase == DivePhase.ASCENDING
    animate_mechanic.emit(mechanic_animation_data)


func _compute_depth_normal() -> float:
    var depth_normal: float = clampf(
        -current_depth_position / absf(dive_data.MAX_DIVE_DEPTH), 0.0, 1.0
    )
    return depth_normal


func _handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.DIVE or state == MechanicController.STATE.DIVE_ASCEND
