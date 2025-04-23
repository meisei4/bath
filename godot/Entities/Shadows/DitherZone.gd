extends Area2D
class_name DitherZone

var friction_factor: float = 0.25
var collision_shape: CollisionShape2D


func _init():
    collision_shape = CollisionShape2D.new()
    var shape: RectangleShape2D = RectangleShape2D.new()
    collision_shape.shape = shape
    add_child(collision_shape)
    monitoring = true
    monitorable = true
    set_collision_layer_value(1, true)
    set_collision_mask_value(1, true)


func set_zone_bounds(bounds: Vector2) -> void:
    var shape: RectangleShape2D = collision_shape.shape as RectangleShape2D
    shape.extents = bounds * 0.5
    collision_shape.position = Vector2.ZERO


func get_friction_factor(global_pos: Vector2) -> float:
    return friction_factor
