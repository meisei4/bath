extends Node2D
class_name MechanicsTest


func _ready() -> void:
    var capsule_scene: PackedScene = (
        ResourceLoader.load("res://godot/Entities/CapsuleDummy.tscn") as PackedScene
    )
    var character: CharacterBody2D = capsule_scene.instantiate() as CharacterBody2D
    character.position = get_viewport().get_visible_rect().size / 2
    add_child(character)
