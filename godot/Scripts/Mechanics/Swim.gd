extends Mechanic
class_name Swim

signal animate_swim(
    current_depth_position: float, depth_normal: float, ascending: bool, frame_delta: float
)

enum DivePhase { LEVEL, ASCENDING, DIVING }
var current_phase: DivePhase = DivePhase.LEVEL

const LEVEL_DEPTH: float = 0.0
const MAX_DIVE_DEPTH: float = -1.0
const DEPTH_SPEED: float = 8.0

var current_depth_position: float = LEVEL_DEPTH
var target_depth_position: float = LEVEL_DEPTH

var debug_autoswim: bool = true
const _DEBUG_PERIOD: float = 1.0
var _debug_clock: float = 0.0

var in_queued_ascend: bool = false

var animation: SwimAnimation


func _ready() -> void:
    super._ready()
    type = Mechanic.TYPE.SWIM
    animation = SwimAnimation.new()
    set_physics_process(false)


func _on_state_changed(state: MechanicController.STATE) -> void:
    if handles_state(state):
        set_physics_process(true)
        if !animate_swim.is_connected(animation.process_animation):
            #TODO: this avoids reconnnection during the ascend state, not good but it works...
            animate_swim.connect(
                animation.process_animation.bind(mechanic_controller.controller_host)
            )

    in_queued_ascend = (state == MechanicController.STATE.SWIM_ASCEND)
    if in_queued_ascend:
        target_depth_position = LEVEL_DEPTH
        _set_phase(DivePhase.ASCENDING)


func _physics_process(delta: float) -> void:
    if debug_autoswim and !in_queued_ascend:
        _debug_clock += delta
        if _debug_clock >= _DEBUG_PERIOD:
            _debug_clock -= _DEBUG_PERIOD
            target_depth_position = (
                MAX_DIVE_DEPTH if target_depth_position == LEVEL_DEPTH else LEVEL_DEPTH
            )

    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _update_depth(time_scaled_delta)
    update_collision()
    emit_mechanic_data(delta)


func _update_depth(delta: float) -> void:
    var step: float = DEPTH_SPEED * delta
    current_depth_position = move_toward(current_depth_position, target_depth_position, step)

    const THRESHOLD: float = 0.001
    if abs(current_depth_position - LEVEL_DEPTH) < THRESHOLD:
        if in_queued_ascend:
            in_queued_ascend = false
            set_physics_process(false)
            animate_swim.disconnect(animation.process_animation)
            state_completed.emit(MechanicController.STATE.SWIM_ASCEND)

        _set_phase(DivePhase.LEVEL)
        return

    _set_phase(DivePhase.ASCENDING if target_depth_position == LEVEL_DEPTH else DivePhase.DIVING)


func emit_mechanic_data(_frame_delta: float) -> void:
    var depth_normal: float = clampf(-current_depth_position / absf(MAX_DIVE_DEPTH), 0.0, 1.0)
    animate_swim.emit(current_depth_position, depth_normal, _is_ascending(), _frame_delta)


func _is_diving() -> bool:
    return current_phase == DivePhase.DIVING


func _is_ascending() -> bool:
    return current_phase == DivePhase.ASCENDING


func _is_level() -> bool:
    return current_phase == DivePhase.LEVEL


func _set_phase(new_phase: DivePhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase


func handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.SWIM or state == MechanicController.STATE.SWIM_ASCEND
