extends Node
class_name Mechanic

var character: CharacterBody2D


func process_input(_delta: float) -> void:
    pass

func process_visual_illusion(_delta: float) -> void:
    pass

func get_sprite_for_visual_illusion() -> Sprite2D:
    for child: Node in character.get_children():
        if child is Sprite2D:
            return child
    return null

func get_sprite_for_visual_illusion1() -> Sprite2D:
    return character.get_node("Sprite2D") as Sprite2D
