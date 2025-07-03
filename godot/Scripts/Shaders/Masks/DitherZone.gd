extends Area2D
class_name DitherZone

var friction_factor: float = 0.25
var collision_shape: CollisionShape2D


func _init() -> void:
    collision_shape = CollisionShape2D.new()
    var shape: RectangleShape2D = RectangleShape2D.new()
    collision_shape.shape = shape
    add_child(collision_shape)
    collision_shape.owner = self
    monitoring = true
    monitorable = true
    set_collision_layer_value(1, true)
    set_collision_mask_value(1, true)


func set_zone_bounds(_bounds: Vector2) -> void:
    pass


func get_friction_factor() -> float:
    return friction_factor
