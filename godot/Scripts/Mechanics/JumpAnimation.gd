extends Node
class_name JumpAnimation


func process_animation(
    vertical_position: float,
    altitude_normal: float,
    ascending: bool,
    character_body: CharacterBody2D,
) -> void:
    var sprite_node: Sprite2D = character_body.get_node("Sprite2D")
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(vertical_position)
    sprite_node.position.y = roundi(-vertical_offset_pixels)
    #_update_sprite_scale_continious(sprite_node, altitude_normal)
    _update_sprite_scale_discrete(sprite_node, altitude_normal)
    var sprite_shader_material: ShaderMaterial = sprite_node.material
    sprite_shader_material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_shader_material.set_shader_parameter("iResolution", sprite_node.texture.get_size())
    sprite_shader_material.set_shader_parameter("ascending", ascending)
    var sprite_height: float = sprite_node.texture.get_size().y
    #TODO: quantize the altitude normal is super important to study for later as it controls exactly how many
    # unique warped sprite frames can exist in the animation
    #TODO: the biggest thing left is quantizing such that we can control a hand-drawn looking pixel perfect tilt animation
    altitude_normal = roundf(altitude_normal * sprite_height) / (sprite_height)  #* 2.0)
    sprite_shader_material.set_shader_parameter("altitude_normal", altitude_normal)

    AnimationManager.update_perspective_tilt_mask(
        sprite_node.texture,
        character_body,
        sprite_node.global_position,
        sprite_node.scale,
        altitude_normal,
        ascending
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
