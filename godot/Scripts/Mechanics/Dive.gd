extends Node
class_name Dive

signal animate_dive(
    current_depth_position: float,
    depth_normal: float,
    ascending: bool,
    frame_delta: float,
    sprite: Sprite2D
)

signal state_completed(completed_state: MechanicController.STATE)

var mechanic_controller: MechanicController

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

var animation: DiveAnimation


func _ready() -> void:
    if !mechanic_controller:
        print("no mechanic controller, bad")
        return

    mechanic_controller.state_changed.connect(_on_state_changed)
    animation = DiveAnimation.new()
    set_physics_process(false)


func _on_state_changed(state: MechanicController.STATE) -> void:
    if handles_state(state):
        set_physics_process(true)
        if !animate_dive.is_connected(animation.process_animation):
            #TODO: this avoids reconnnection during the ascend state, not good but it works...
            animate_dive.connect(animation.process_animation)

    in_queued_ascend = (state == MechanicController.STATE.DIVE_ASCEND)
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
    _update_collision()
    _emit_animation_data(delta)


func _update_depth(delta: float) -> void:
    var step: float = DEPTH_SPEED * delta
    current_depth_position = move_toward(current_depth_position, target_depth_position, step)

    const THRESHOLD: float = 0.001
    if abs(current_depth_position - LEVEL_DEPTH) < THRESHOLD:
        if in_queued_ascend:
            in_queued_ascend = false
            set_physics_process(false)
            animate_dive.disconnect(animation.process_animation)
            state_completed.emit(MechanicController.STATE.DIVE_ASCEND)

        _set_phase(DivePhase.LEVEL)
        return

    _set_phase(DivePhase.ASCENDING if target_depth_position == LEVEL_DEPTH else DivePhase.DIVING)


func _is_diving() -> bool:
    return current_phase == DivePhase.DIVING


func _is_ascending() -> bool:
    return current_phase == DivePhase.ASCENDING


func _is_level() -> bool:
    return current_phase == DivePhase.LEVEL


func _set_phase(new_phase: DivePhase) -> void:
    if current_phase != new_phase:
        current_phase = new_phase


func _update_collision() -> void:
    if _is_level():
        mechanic_controller.controller_host.collision_shape.disabled = false  #TODO: lmao double negatives
    else:
        mechanic_controller.controller_host.collision_shape.disabled = true


func _emit_animation_data(_frame_delta: float) -> void:
    var depth_normal: float = clampf(-current_depth_position / absf(MAX_DIVE_DEPTH), 0.0, 1.0)
    animate_dive.emit(
        current_depth_position,
        depth_normal,
        _is_ascending(),
        _frame_delta,
        mechanic_controller.controller_host.sprite
    )
    AnimationManager.update_perspective_tilt_mask(
        mechanic_controller.controller_host.sprite.texture,
        mechanic_controller.controller_host,
        mechanic_controller.controller_host.sprite.global_position,
        mechanic_controller.controller_host.sprite.scale,
        depth_normal,
        _is_ascending()
    )


func handles_state(state: MechanicController.STATE) -> bool:
    return state == MechanicController.STATE.DIVE or state == MechanicController.STATE.DIVE_ASCEND
