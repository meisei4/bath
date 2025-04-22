extends Mechanic
class_name LateralMovement

const MAX_SPEED: float = 300.0
const ACCELERATION: float = 400.0
const DECELERATION: float = 600.0
const LATERAL_FADE_TIME: float = 0.3  # how long (seconds) to reach full wiggle

#TODO: this is a clone of movement_input to be used for animations, i.e. persist between input events
var lateral_dir: int = 0  # –1 for left, +1 for right, 0 for none
var lateral_timer: float = 0  # ramps 0→LATERAL_FADE_TIME while moving, back to 0 when stopped

var movement_input: int = 0
var current_velocity: float = 0.0
var stretch_timer: float = 0.0


func _ready() -> void:
    #apply_mechanic_animation_shader("res://Resources/Shaders/MechanicAnimations/mechanic_animations.gdshader")
    MechanicManager.left_lateral_movement.connect(_on_move_left_triggered)
    MechanicManager.right_lateral_movement.connect(_on_move_right_triggered)


func _process(delta: float) -> void:
    process_input(delta)
    process_visual_illusion(delta)


func _on_move_left_triggered() -> void:
    movement_input = -1
    lateral_dir = -1


func _on_move_right_triggered() -> void:
    movement_input = 1
    lateral_dir = 1


func process_input(delta: float) -> void:
    var time: float = SpacetimeContext.apply_time_scale(delta)
    current_velocity = SpacetimeContext.apply_universal_drag(current_velocity, time)
    _apply_movement_input(time)
    _apply_cosmic_friction(time)
    _move_character(time)

    if lateral_dir != 0:
        lateral_timer = clamp(lateral_timer + delta, 0.0, LATERAL_FADE_TIME)
    else:
        lateral_timer = clamp(lateral_timer - delta, 0.0, LATERAL_FADE_TIME)
    # finally clear movement_input, but leave lateral_dir/timer intact
    movement_input = 0


func process_visual_illusion(_delta: float) -> void:
    var sprite_node: Sprite2D = get_sprite_for_visual_illusion()
    # Figure out a [0…1] phase based on the timer:
    var phase = lateral_timer / LATERAL_FADE_TIME
    # Combine with direction for a signed [–1…+1] value:
    var lateral_amount_normal = lateral_dir * phase
    sprite_node.material.set_shader_parameter("lateral_amount_normal", lateral_amount_normal)


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
    var friction_amount: float = SpacetimeContext.COSMIC_FRICTION * time
    if current_velocity > 0.0:
        current_velocity = max(0.0, current_velocity - friction_amount)
    elif current_velocity < 0.0:
        current_velocity = min(0.0, current_velocity + friction_amount)


func _move_character(time: float) -> void:
    var delta_world_units: float = current_velocity * time
    var delta_pixels: float = SpacetimeContext.to_physical_space(delta_world_units)
    character.position.x += delta_pixels
