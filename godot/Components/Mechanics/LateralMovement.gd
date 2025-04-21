extends Mechanic
class_name LateralMovement


const MAX_SPEED:           float = 300.0
const ACCELERATION:        float = 400.0
const DECELERATION:        float = 600.0
const PEAK_STRETCH_SCALE:  float = 8.0
const STRETCH_DURATION:    float = 0.2   # time to reach full stretch


var movement_input:    int   = 0
var current_velocity:  float = 0.0
var stretch_timer:     float = 0.0

func _ready() -> void:
    MechanicManager.left_lateral_movement .connect(_on_move_left_triggered)
    MechanicManager.right_lateral_movement.connect(_on_move_right_triggered)

func _process(delta: float) -> void:
    process_input(delta)
    process_visual_illusion(delta)

func _on_move_left_triggered() -> void:
    movement_input = -1

func _on_move_right_triggered() -> void:
    movement_input = 1

func process_input(delta: float) -> void:
    var time: float = SpacetimeContext.apply_time_scale(delta)
    current_velocity = SpacetimeContext.apply_universal_drag(current_velocity, time)
    _apply_movement_input(time)
    _apply_cosmic_friction(time)
    _move_character(time)

    if movement_input != 0:
        stretch_timer = clamp(stretch_timer + time, 0.0, STRETCH_DURATION)
    else:
        stretch_timer = clamp(stretch_timer - time, 0.0, STRETCH_DURATION)

    movement_input = 0

func process_visual_illusion(_delta: float) -> void:
    var sprite_node: Sprite2D = get_sprite_for_visual_illusion()

    var time_ratio:float = (stretch_timer / STRETCH_DURATION) if STRETCH_DURATION > 0.0 else 0.0
    var stretch_x:     float = lerp(1.0, PEAK_STRETCH_SCALE, time_ratio)

    # compose X‐stretch with current uniform (jump) scale!!!!!!!! (additive scalars break the composed scaling effects
    # Vector2(stretch_x,1) * sprite_node.scale multiplies element‐wise:
    #   new_x = stretch_x * old_x
    #   new_y = 1     * old_y
    sprite_node.scale = Vector2(stretch_x, 1.0) * sprite_node.scale


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
    var delta_pixels:      float = SpacetimeContext.to_physical_space(delta_world_units)
    character.position.x += delta_pixels
