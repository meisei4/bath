extends Node
#class_name ComputeShaderSignalManager

var collision_mask: CollisionMask
var glacier_flow: GlacierFlow
var perspective_tilt_mask: PerspectiveTiltMask
var character_bodies: Array[CharacterBody2D]
var umbral_shadow: ShaderMaterial

signal iTime_update(iTime: float)

signal visual_illusion_updated(
    sprite_index: int,
    center_px: Vector2,
    half_size_px: Vector2,
    altitude_normal: float,
    ascending: float
)


func register_perspective_tilt_mask(_perspective_tilt_mask: PerspectiveTiltMask) -> void:
    if (
        self.perspective_tilt_mask
        and self.visual_illusion_updated.is_connected(_on_visual_illusion_updated)
    ):
        self.visual_illusion_updated.disconnect(_on_visual_illusion_updated)

    self.visual_illusion_updated.connect(_on_visual_illusion_updated)

    self.perspective_tilt_mask = _perspective_tilt_mask
    for character_body: CharacterBody2D in character_bodies:
        _configure_character_body(character_body as CapsuleDummy)


func register_umbral_shadow(_umbral_shadow: ShaderMaterial) -> void:
    self.umbral_shadow = _umbral_shadow
    #TODO: turn this into a signal handling, to be more clear that this class is actually for that
    if perspective_tilt_mask:
        umbral_shadow.set_shader_parameter(
            "iChannel1", perspective_tilt_mask.perspective_tilt_mask_texture
        )


func register_character_body(_character_body: CharacterBody2D) -> void:
    character_bodies.append(_character_body)
    _configure_character_body(_character_body as CapsuleDummy)


func _configure_character_body(_character_body: CapsuleDummy) -> void:
    if !perspective_tilt_mask:
        return
    var sprite_node: Sprite2D = _character_body.get_node("Sprite2D") as Sprite2D
    var tex: Texture2D = sprite_node.texture
    var index: int = perspective_tilt_mask.register_sprite_texture(tex)
    for mechanic_type: Mechanic.TYPE in _character_body.mechanics.keys():
        match mechanic_type:
            Mechanic.TYPE.JUMP:
                _character_body.mechanics[mechanic_type].sprite_texture_index = index
                break


func register_collision_mask(_collision_mask: CollisionMask) -> void:
    self.collision_mask = _collision_mask


func register_glacier_flow(_glacier_flow: GlacierFlow) -> void:
    if self.glacier_flow and self.iTime_update.is_connected(_on_iTime_update):
        self.iTime_update.disconnect(_on_iTime_update)

    self.glacier_flow = _glacier_flow
    self.iTime_update.connect(_on_iTime_update)


func _on_iTime_update(iTime: float) -> void:
    if collision_mask:
        collision_mask.iTime = iTime


func _on_visual_illusion_updated(
    sprite_index: int,
    center_px: Vector2,
    half_size_px: Vector2,
    altitude_normal: float,
    ascending: float
) -> void:
    perspective_tilt_mask.update_cpu_side_sprite_data_ssbo_cache(
        sprite_index, center_px, half_size_px, altitude_normal, ascending
    )
