extends Node2D
class_name MechanicsTest

var capsule_dummy: CapsuleDummy
var capsule_scene: PackedScene = preload("res://TestScenes/Entities/Characters/CapsuleDummy.tscn")


func _ready() -> void:
    capsule_dummy = capsule_scene.instantiate() as CapsuleDummy
    capsule_dummy.z_index = 1
    add_child(capsule_dummy)
    var viewport_size: Vector2 = ResolutionManager.resolution
    var sprite_node: Sprite2D = capsule_dummy.get_node("Sprite2D") as Sprite2D
    var sprite_size: Vector2 = sprite_node.texture.get_size()
    capsule_dummy.position = Vector2(viewport_size.x * 0.5, viewport_size.y - sprite_size.y * 0.5)
    MechanicManager.register_controller(capsule_dummy)
