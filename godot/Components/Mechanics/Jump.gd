extends Mechanic
class_name Jump

@export var initial_jump_velocity: float = 10.0  # in "world units"/s
@export var override_gravity: float = 0.0  # if > 0, use that instead of SpacetimeContext.GRAVITY

var vertical_velocity: float = 0.0


func _ready() -> void:
    MechanicManager.jump.connect(_on_jump)


func _on_jump() -> void:
    vertical_velocity = -initial_jump_velocity


func process_input(real_delta: float) -> void:
    var scaled_delta: float = SpacetimeContext.apply_time_scale(real_delta)

    var g: float = override_gravity if (override_gravity > 0.0) else SpacetimeContext.GRAVITY
    vertical_velocity += g * scaled_delta

    vertical_velocity = SpacetimeContext.apply_universal_drag(vertical_velocity, scaled_delta)

    if SpacetimeContext.random_quantum_tunnel_check():
        vertical_velocity = -vertical_velocity * 0.5

    var delta_world_units: float = vertical_velocity * scaled_delta
    var delta_pixels: float = SpacetimeContext.to_physical_space(delta_world_units)
    character.position.y += delta_pixels

    if vertical_velocity >= 0.0:
        vertical_velocity = 0.0
