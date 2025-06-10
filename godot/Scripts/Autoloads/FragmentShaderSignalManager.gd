extends Node
#class_name FragmentShaderSignalManager

var ice_sheets: IceSheets
var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
var character_bodies: Array[CharacterBody2D]
var umbral_shadow: ShaderMaterial

signal visual_illusion_updated(
    sprite_index: int,
    center_px: Vector2,
    half_size_px: Vector2,
    altitude_normal: float,
    ascending: float
)


func register_perspective_tilt_mask_fragment(
    _perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
) -> void:
    if (
        self.perspective_tilt_mask_fragment
        and self.visual_illusion_updated.is_connected(_on_visual_illusion_updated_fragment)
    ):
        self.visual_illusion_updated.disconnect(_on_visual_illusion_updated_fragment)
    perspective_tilt_mask_fragment = _perspective_tilt_mask_fragment
    visual_illusion_updated.connect(_on_visual_illusion_updated_fragment)


func register_umbral_shadow_fragment(_umbral_shadow: ShaderMaterial) -> void:
    self.umbral_shadow = _umbral_shadow
    if perspective_tilt_mask_fragment:
        umbral_shadow.set_shader_parameter(
            "iChannel1", perspective_tilt_mask_fragment.get_perspective_tilt_mask_texture_fragment()
        )


func register_character_body_fragment(_character_body: CharacterBody2D) -> void:
    character_bodies.append(_character_body)
    _configure_character_body_fragment(_character_body as CapsuleDummy)


func _configure_character_body_fragment(_character_body: CapsuleDummy) -> void:
    if !perspective_tilt_mask_fragment:
        return
    var sprite_node: Sprite2D = _character_body.get_node("Sprite2D") as Sprite2D
    var sprite_texture: Texture2D = sprite_node.texture
    var index: int = perspective_tilt_mask_fragment.register_sprite_texture(sprite_texture)
    for mechanic_type: Mechanic.TYPE in _character_body.mechanics.keys():
        match mechanic_type:
            Mechanic.TYPE.JUMP:
                _character_body.mechanics[mechanic_type].sprite_texture_index = index
                break


func register_collision_mask_fragment(_collision_mask_fragment: CollisionMaskFragment) -> void:
    self.collision_mask_fragment = _collision_mask_fragment


func register_ice_sheets_fragment(_ice_sheets: IceSheets) -> void:
    self.ice_sheets = _ice_sheets


func _on_visual_illusion_updated_fragment(
    sprite_index: int,
    center_px: Vector2,
    sprite_scale: Vector2,
    altitude_normal: float,
    ascending: float
) -> void:
    var sprite_texture: Texture2D = character_bodies[sprite_index].get_node("Sprite2D").texture
    perspective_tilt_mask_fragment.set_sprite_data(
        sprite_texture, sprite_index, center_px, sprite_scale, altitude_normal, ascending
    )
