extends Node
#class_name MaskManager

signal ice_sheets_entered_scene(ice_sheets: IceSheetsRenderer)

var iTime: float
var ice_sheets: IceSheetsRenderer
var umbral_shadow: ShaderMaterial
var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
var sprite_to_mask_index: Dictionary[Sprite2D, int]


func register_ice_sheets(_ice_sheets: IceSheetsRenderer) -> void:
    self.ice_sheets = _ice_sheets
    ice_sheets_entered_scene.emit(_ice_sheets)


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
        sprite, sprite_to_mask_index[sprite], altitude_normal, 1.0 if ascending else 0.0
    )


func update_perspective_tilt_mask_sprite_rotation(
    sprite: Sprite2D,
) -> void:
    if !perspective_tilt_mask_fragment:
        return
    perspective_tilt_mask_fragment.set_sprite_rotation(sprite, sprite_to_mask_index[sprite])


func register_umbral_shadow(_umbral_shadow: ShaderMaterial) -> void:
    self.umbral_shadow = _umbral_shadow
    if perspective_tilt_mask_fragment:
        umbral_shadow.set_shader_parameter(
            "iChannel1", perspective_tilt_mask_fragment.get_perspective_tilt_mask_texture_fragment()
        )
