extends Node
class_name Mechanic

var character: CharacterBody2D
var animation_shader: ShaderMaterial


func process_input(_delta: float) -> void:
    pass


func process_visual_illusion(_delta: float) -> void:
    pass


func process_collision_shape(_delta: float) -> void:
    pass


func apply_mechanic_animation_shader(shader_path: String) -> void:
    var sprite: Sprite2D = get_sprite_for_visual_illusion()
    var shader_material: ShaderMaterial = ShaderMaterial.new()
    shader_material.shader = load(shader_path)
    sprite.material = shader_material
    animation_shader = shader_material


func get_sprite_for_visual_illusion() -> Sprite2D:
    for child: Node in character.get_children():
        if child is Sprite2D:
            return child
    return null


func get_collision_object_for_processing() -> CollisionShape2D:
    for child: Node in character.get_children():
        if child is CollisionShape2D:
            return child
    return null
