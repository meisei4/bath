extends Mechanic
class_name LateralMovement

#TODO: this whole mechanic has a lot to learn from the Jump refactoring
const MAX_SPEED: float = 300.0
const ACCELERATION: float = 400.0
const DECELERATION: float = 600.0

var movement_input: int = 0
var current_velocity: float = 0.0
var stretch_timer: float = 0.0


func _ready() -> void:
    #apply_mechanic_animation_shader("res://Resources/Shaders/MechanicAnimations/mechanic_animations.gdshader")
    #TODO: these signals and the mechanics manager direct calls to process input are hurting my head, I think things are being ran twice per frame?
    MechanicManager.left_lateral_movement.connect(_on_move_left_triggered)
    MechanicManager.right_lateral_movement.connect(_on_move_right_triggered)


func _on_move_left_triggered() -> void:
    movement_input = -1


func _on_move_right_triggered() -> void:
    movement_input = 1


func process_input(delta: float) -> void:
    var time: float = SpacetimeManager.apply_time_scale(delta)
    current_velocity = SpacetimeManager.apply_universal_drag(current_velocity, time)
    _apply_movement_input(time)
    _apply_cosmic_friction(time)
    _move_character(time)
    movement_input = 0


func process_visual_illusion(_delta: float) -> void:
    pass


func _apply_movement_input(time: float) -> void:
    if movement_input != 0:
        current_velocity += ACCELERATION * time * float(movement_input)
        current_velocity = clamp(current_velocity, -MAX_SPEED, MAX_SPEED)
    else:
        if current_velocity > 0.0:
            current_velocity = max(0.0, current_velocity - DECELERATION * time)
        elif current_velocity < 0.0:
            current_velocity = min(0.0, current_velocity + DECELERATION * time)


func _apply_cosmic_friction(time: float) -> void:
    var friction_amount: float = SpacetimeManager.COSMIC_FRICTION * time
    if current_velocity > 0.0:
        current_velocity = max(0.0, current_velocity - friction_amount)
    elif current_velocity < 0.0:
        current_velocity = min(0.0, current_velocity + friction_amount)


func _move_character(time: float) -> void:
    var delta_world_units: float = current_velocity * time
    var delta_pixels: float = SpacetimeManager.to_physical_space(delta_world_units)
    character_body.position.x += delta_pixels
