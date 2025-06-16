extends Node
class_name FlipAnimation

var sprite: Sprite2D


func _ready() -> void:
    if !sprite:
        print("no sprite, bad")
        return
   #if sprite.material == null:
        #sprite.material = ShaderMaterial.new()


func process_animation_data(mechanic_animation_data: MechanicAnimationData) -> void:
    var rotation_normal: float = mechanic_animation_data.vertical_normal
    _update_sprite_rotation(sprite, rotation_normal)
    MaskManager.update_perspective_tilt_mask_sprite_rotation(sprite)


func _update_sprite_rotation(sprite: Sprite2D, flip_normal: float) -> void:
    sprite.rotation = flip_normal * TAU
