extends Node
#class_name MechanicAnimationsManager

signal jump(character_index: int, vertical_position: float, altitude_normal: float, ascending: bool)

signal swim(
    character_index: int,
    current_depth_position: float,
    depth_normal: float,
    ascending: bool,
    frame_delta: float
)

var MECHANICS_ANIMATIONS: Dictionary[Mechanic.TYPE, Shader] = {
    Mechanic.TYPE.JUMP: preload(ResourcePaths.JUMP_TRIG_SHADER),
    Mechanic.TYPE.SWIM: preload(ResourcePaths.SWIM_SHADER),
}

var active_shader: Shader
var umbral_shadow: ShaderMaterial
var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment


func _ready() -> void:
    jump.connect(process_jump_animation)
    swim.connect(process_swim_animation)
    MechanicsManager.character_body_registered.connect(_on_character_body_registered)
    MechanicsManager.active_mechanic_changed.connect(_on_active_mechanic_changed)
    set_process(false)


func process_jump_animation(
    character_index: int, vertical_position: float, altitude_normal: float, ascending: bool
) -> void:
    var character_body: CharacterBody2D = MechanicsManager.character_bodies[character_index]
    var sprite_node: Sprite2D = character_body.get_node("Sprite2D")
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(vertical_position)
    sprite_node.position.y = roundi(-vertical_offset_pixels)
    #_update_sprite_scale_continious(sprite_node, altitude_normal)
    _update_sprite_scale_discrete(sprite_node, altitude_normal)
    sprite_node.material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_node.material.set_shader_parameter("iResolution", sprite_node.texture.get_size())
    sprite_node.material.set_shader_parameter("ascending", ascending)
    var sprite_height: float = sprite_node.texture.get_size().y
    #TODO: quantize the altitude normal is super important to study for later as it controls exactly how many
    # unique warped sprite frames can exist in the animation
    #TODO: the biggest thing left is quantizing such that we can control a hand-drawn looking pixel perfect tilt animation
    altitude_normal = roundf(altitude_normal * sprite_height) / (sprite_height)  #* 2.0)
    sprite_node.material.set_shader_parameter("altitude_normal", altitude_normal)
    perspective_tilt_mask_fragment.set_sprite_data(
        sprite_node.texture,
        character_index,
        sprite_node.global_position,
        sprite_node.scale,
        altitude_normal,
        1.0 if ascending else 0.0
    )


var SPRITE_SCALE_AT_MIN_ALTITUDE: float = 1.0
var SPRITE_SCALE_AT_MAX_ALTITUDE: float = 3.0


func _update_sprite_scale_continious(sprite_node: Sprite2D, _altitude_normal: float) -> void:
    var scale_minimum: float = SPRITE_SCALE_AT_MIN_ALTITUDE
    var scale_maximum: float = SPRITE_SCALE_AT_MAX_ALTITUDE
    var scale_multiplier: float = scale_minimum + (scale_maximum - scale_minimum) * _altitude_normal
    sprite_node.scale = Vector2.ONE * scale_multiplier


func _eased_phase(_altitude_normal: float) -> float:
    const EASE_EXP: float = 2.0
    return pow(_altitude_normal, EASE_EXP)


func _update_sprite_scale_discrete(sprite_node: Sprite2D, _altitude_normal: float) -> void:
    var base_width_i: int = int(sprite_node.texture.get_size().x)
    var base_height_i: int = int(sprite_node.texture.get_size().y)
    var scale_minimum_f: float = SPRITE_SCALE_AT_MIN_ALTITUDE
    var scale_maximum_f: float = SPRITE_SCALE_AT_MAX_ALTITUDE
    var eased_altitude_normal: float = _eased_phase(_altitude_normal)
    #var interpolated_scale_f: float = lerp(scale_minimum_f, scale_maximum_f, altitude_location)
    #TODO: below is an explicit lerp function
    var interpolated_scale_f: float = (
        scale_minimum_f + (scale_maximum_f - scale_minimum_f) * eased_altitude_normal
    )
    # Example (base 16×24), min scale = 1.0, max scale = 3.0
    # altitude_normal | interp_scale_f |  gcd_uniform  | scale_mult_f | final dimensions
    # -----------------------------------------------------------------------------------
    # 0.00            | 1.0            | 8             | 1.0          | (16, 24)
    # 0.25            | 1.5            | 12            | 1.5          | (24, 36)
    # 0.50            | 2.0            | 16            | 2.0          | (32, 48)
    # 0.75            | 2.5            | 20            | 2.5          | (40, 60)
    # 1.00            | 3.0            | 24            | 3.0          | (48, 72)
    var temp_a: int = base_width_i
    var temp_b: int = base_height_i
    while temp_b != 0:
        var remainder: int = temp_a % temp_b
        temp_a = temp_b
        temp_b = remainder
    var greatest_common_divisor_uniform: int = temp_a
    var steps_i: int = greatest_common_divisor_uniform
    var quantized_steps_i: int = roundi(interpolated_scale_f * steps_i)
    var scale_multiplier_f: float = quantized_steps_i / float(steps_i)
    sprite_node.scale = Vector2.ONE * scale_multiplier_f


const MAX_DIVE_DEPTH: float = -1.0


func process_swim_animation(
    character_index: int,
    current_depth_position: float,
    depth_normal: float,
    ascending: bool,
    frame_delta: float
) -> void:
    var character_body: CharacterBody2D = MechanicsManager.character_bodies[character_index]
    var sprite_node: Sprite2D = character_body.get_node("Sprite2D")
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(current_depth_position)
    sprite_node.position.y = -vertical_offset_pixels
    sprite_node.material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_node.material.set_shader_parameter("ascending", ascending)
    sprite_node.material.set_shader_parameter("depth_normal", depth_normal)
    _update_sprite_scale(sprite_node, depth_normal, frame_delta)
    perspective_tilt_mask_fragment.set_sprite_data(
        sprite_node.texture,
        character_index,
        sprite_node.global_position,
        sprite_node.scale,
        depth_normal,
        1.0 if ascending else 0.0
    )


func _update_sprite_scale(sprite: Sprite2D, depth_normal: float, _frame_delta: float) -> void:
    var scale_min: float = 0.5
    var scale_max: float = 1.0
    var smooth_depth: float = smoothstep(0.0, 1.0, depth_normal)
    sprite.scale = Vector2.ONE * lerp(scale_max, scale_min, smooth_depth)


func register_umbral_shadow(_umbral_shadow: ShaderMaterial) -> void:
    self.umbral_shadow = _umbral_shadow
    if perspective_tilt_mask_fragment:
        umbral_shadow.set_shader_parameter(
            "iChannel1", perspective_tilt_mask_fragment.get_perspective_tilt_mask_texture_fragment()
        )


func register_perspective_tilt_mask_fragment(
    _perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
) -> void:
    perspective_tilt_mask_fragment = _perspective_tilt_mask_fragment


func _on_character_body_registered(_character_body: CharacterBody2D) -> void:
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


func _on_active_mechanic_changed(mechanic: Mechanic) -> void:
    var shader: Shader = MECHANICS_ANIMATIONS[mechanic.mechanic_type]
    if shader == null or shader == active_shader:
        return

    var sprite: Sprite2D = mechanic.character_body.get_node("Sprite2D") as Sprite2D
    if sprite:
        if sprite.material:
            sprite.material.shader = shader
            active_shader = shader
        else:
            sprite.material = ShaderMaterial.new()
