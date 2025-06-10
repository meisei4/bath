extends Area2D
class_name UmbralZone

var cooling_factor: float = 0.5
var collision_shape: CollisionShape2D


func _init() -> void:
    collision_shape = CollisionShape2D.new()
    var shape: RectangleShape2D = RectangleShape2D.new()
    collision_shape.shape = shape
    add_child(collision_shape)
    monitoring = true
    monitorable = true
    set_collision_layer_value(1, true)
    set_collision_mask_value(1, true)


func set_zone_bounds(_bounds: Vector2) -> void:
    pass


func get_cooling_factor() -> float:
    return cooling_factor
