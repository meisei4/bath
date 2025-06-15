extends Node
#class_name AnimationManager

var umbral_shadow: ShaderMaterial
var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
var character_body_to_mask_index: Dictionary[CharacterBody2D, int]
var sprite_to_mask_index: Dictionary[Sprite2D, int]


func register_perspective_tilt_mask_fragment(
    _perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
) -> void:
    perspective_tilt_mask_fragment = _perspective_tilt_mask_fragment


func update_perspective_tilt_mask(
    sprite: Sprite2D, altitude_normal: float, ascending: bool
) -> void:
    if !perspective_tilt_mask_fragment:
        return
    perspective_tilt_mask_fragment.set_sprite_data(
        sprite.texture,
        sprite_to_mask_index[sprite],
        sprite.global_position,
        sprite.scale,
        altitude_normal,
        1.0 if ascending else 0.0
    )


func update_perspective_tilt_mask1(
    sprite_texture: Texture2D,
    character_body: CharacterBody2D,
    sprite_position: Vector2,
    sprite_scale: Vector2,
    altitude_normal: float,
    ascending: bool
) -> void:
    if !perspective_tilt_mask_fragment:
        return
    perspective_tilt_mask_fragment.set_sprite_data(
        sprite_texture,
        character_body_to_mask_index[character_body],
        sprite_position,
        sprite_scale,
        altitude_normal,
        1.0 if ascending else 0.0
    )


func register_umbral_shadow(_umbral_shadow: ShaderMaterial) -> void:
    self.umbral_shadow = _umbral_shadow
    if perspective_tilt_mask_fragment:
        umbral_shadow.set_shader_parameter(
            "iChannel1", perspective_tilt_mask_fragment.get_perspective_tilt_mask_texture_fragment()
        )
