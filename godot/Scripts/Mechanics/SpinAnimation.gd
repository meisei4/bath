extends Node
class_name SpinAnimation

var sprite: Sprite2D


func _ready() -> void:
    if !sprite:
        print("no sprite, bad")
        return


func process_animation_data(mechanic_animation_data: MechanicAnimationData) -> void:
    var rotation_normal: float = mechanic_animation_data.vertical_normal
    _update_sprite_rotation(sprite, rotation_normal)
    MaskManager.update_perspective_tilt_mask_sprite_rotation(sprite)


func _update_sprite_rotation(sprite: Sprite2D, spin_normal: float) -> void:
    sprite.rotation = spin_normal * TAU
