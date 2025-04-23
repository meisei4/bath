extends Node2D
class_name MechanicsTest


func _ready() -> void:
    var capsule_scene: PackedScene = (
        ResourceLoader.load("res://godot/Test/Mechanics/CapsuleDummy.tscn") as PackedScene
    )
    var character: CharacterBody2D = capsule_scene.instantiate() as CharacterBody2D
    character.z_index = 0
    add_child(character)

    var viewport_size: Vector2 = get_viewport().get_visible_rect().size

    var sprite: Sprite2D = character.get_node("Sprite2D") as Sprite2D
    var sprite_size: Vector2 = sprite.texture.get_size()

    character.position = Vector2(viewport_size.x * 0.5, viewport_size.y - sprite_size.y * 0.5)
