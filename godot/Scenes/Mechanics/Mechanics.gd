extends Node2D
class_name Mechanics

@onready var capsule_dummy: CapsuleDummy = $CharacterBody2D
@onready var capsule_dummy_scene: PackedScene = preload(ResourcePaths.CAPSULE_DUMMY)

#@export var capsule_dummy: CapsuleDummy
#@export var capsule_dummy_scene: PackedScene = preload(ResourcePaths.CAPSULE_DUMMY)

func _ready() -> void:
    capsule_dummy = capsule_dummy_scene.instantiate()
    capsule_dummy.z_index = 1
    add_child(capsule_dummy)
    var viewport_size: Vector2 = ResolutionManager.resolution
    var sprite_node: Sprite2D = capsule_dummy.get_node("Sprite2D")
    var sprite_size: Vector2 = sprite_node.texture.get_size()
    capsule_dummy.position = Vector2(viewport_size.x * 0.5, viewport_size.y - sprite_size.y * 0.5)
