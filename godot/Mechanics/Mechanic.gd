extends Node
class_name Mechanic

var character_body: CapsuleDummy
var animation_shader: ShaderMaterial
var sprite_texture_index: int  #TODO: this is hacked, idk cant tell until having multiple sprites

var visuals_enabled: bool = true


func process_input(_delta: float) -> void:
    pass


func process_visual_illusion(_delta: float) -> void:
    pass


func process_collision_shape(_delta: float) -> void:
    pass


func apply_mechanic_animation_shader(_shader: Shader) -> void:
    var sprite: Sprite2D = get_sprite_for_visual_illusion()
    if animation_shader == null:
        animation_shader = ShaderMaterial.new()
        animation_shader.shader = _shader

    sprite.material = animation_shader


func get_sprite_for_visual_illusion() -> Sprite2D:
    for child: Node in character_body.get_children():
        if child is Sprite2D:
            return child
    return null


func get_collision_object_for_processing() -> CollisionShape2D:
    for child: Node in character_body.get_children():
        if child is CollisionShape2D:
            return child
    return null
