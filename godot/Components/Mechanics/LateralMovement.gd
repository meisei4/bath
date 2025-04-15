extends Mechanic
class_name LateralMovement

@export var max_speed: float = 300.0  # In "world units" per second (example)
@export var acceleration: float = 800.0  # In "world units" / s^2 (example)
@export var deceleration: float = 600.0  # In "world units" / s^2 (example)

var current_velocity: float = 0.0
var target_direction: int = 0


func _ready() -> void:
    MechanicManager.left_lateral_movement.connect(_on_left)
    MechanicManager.right_lateral_movement.connect(_on_right)


func _on_left() -> void:
    target_direction = -1


func _on_right() -> void:
    target_direction = 1


func process_input(real_delta: float) -> void:
    var scaled_delta: float = SpacetimeContext.apply_time_scale(real_delta)
    current_velocity = SpacetimeContext.apply_universal_drag(current_velocity, scaled_delta)

    if target_direction != 0:
        current_velocity += acceleration * scaled_delta * float(target_direction)
        current_velocity = clamp(current_velocity, -max_speed, max_speed)
    else:
        if current_velocity > 0.0:
            current_velocity = max(0.0, current_velocity - deceleration * scaled_delta)
        elif current_velocity < 0.0:
            current_velocity = min(0.0, current_velocity + deceleration * scaled_delta)

    if SpacetimeContext.COSMIC_FRICTION > 0.0:
        if abs(current_velocity) > 0.01:
            var friction_amount = SpacetimeContext.COSMIC_FRICTION * scaled_delta
            if current_velocity > 0.0:
                current_velocity = max(0.0, current_velocity - friction_amount)
            else:
                current_velocity = min(0.0, current_velocity + friction_amount)

    if SpacetimeContext.random_quantum_tunnel_check():
        current_velocity = -current_velocity

    var delta_world_units: float = current_velocity * scaled_delta
    var delta_pixels: float = SpacetimeContext.to_physical_space(delta_world_units)
    character.position.x += delta_pixels

    target_direction = 0
