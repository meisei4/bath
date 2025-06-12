extends Mechanic
class_name Swim

signal animate_swim(
    current_depth_position: float, depth_normal: float, ascending: bool, frame_delta: float
)

enum DivePhase { LEVEL, ASCENDING, DIVING }  #TODO: i dont want to add an APEX_FLOAT phase but maybe...
var current_phase: DivePhase = DivePhase.LEVEL

const LEVEL_DEPTH: float = 0.0
const MAX_DIVE_DEPTH: float = -1.0
const DEPTH_SPEED: float = 8.0

var current_depth_position: float = LEVEL_DEPTH
var target_depth_position: float = LEVEL_DEPTH

var debug_autoswim: bool = false
const _DEBUG_PERIOD: float = 1.0
var _debug_clock: float = 0.0


func _ready() -> void:
    type = Mechanic.TYPE.SWIM


func _process(delta: float) -> void:
    if !debug_autoswim:
        return
    _debug_clock += delta
    if _debug_clock >= _DEBUG_PERIOD:
        _debug_clock -= _DEBUG_PERIOD
        target_depth_position = (
            MAX_DIVE_DEPTH if target_depth_position == LEVEL_DEPTH else LEVEL_DEPTH
        )


func update_position_delta_pixels(delta: float) -> void:
    var time_scaled_delta: float = SpacetimeManager.apply_time_scale(delta)
    _update_depth(time_scaled_delta)


func _update_depth(delta: float) -> void:
    var step: float = DEPTH_SPEED * delta
    current_depth_position = move_toward(current_depth_position, target_depth_position, step)
    const THRESHOLD: float = 0.001
    if abs(current_depth_position - LEVEL_DEPTH) < THRESHOLD:
        _set_phase(DivePhase.LEVEL)
        return
    _set_phase(DivePhase.ASCENDING if target_depth_position == LEVEL_DEPTH else DivePhase.DIVING)


func emit_mechanic_data(_frame_delta: float) -> void:
    var depth_normal: float = clampf(-current_depth_position / absf(MAX_DIVE_DEPTH), 0.0, 1.0)
    animate_swim.emit(current_depth_position, depth_normal, is_ascending(), _frame_delta)


func is_diving() -> bool:
    return current_phase == DivePhase.DIVING


func is_ascending() -> bool:
    return current_phase == DivePhase.ASCENDING


func is_level() -> bool:
    return current_phase == DivePhase.LEVEL


func _set_phase(new_phase: DivePhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase


func handles_state(state: MechanicManager.STATE) -> bool:
    return state == MechanicManager.State.SWIM or state == MechanicManager.State.SWIM_ASCEND
