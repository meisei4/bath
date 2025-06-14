extends Node
class_name SwimAnimation

var shader: Shader = preload(ResourcePaths.SWIM_SHADER)


func process_animation(
    current_depth_position: float,
    depth_normal: float,
    ascending: bool,
    frame_delta: float,
    character_body: CharacterBody2D
) -> void:
    var sprite_node: Sprite2D = character_body.get_node("Sprite2D")
    if sprite_node.material == null:
        sprite_node.material = ShaderMaterial.new()
    if sprite_node.material.shader != self.shader:
        sprite_node.material.shader = self.shader
    var sprite_shader_material: ShaderMaterial = sprite_node.material
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(current_depth_position)
    sprite_node.position.y = -vertical_offset_pixels
    sprite_shader_material.set_shader_parameter("iChannel0", sprite_node.texture)
    sprite_shader_material.set_shader_parameter("ascending", ascending)
    sprite_shader_material.set_shader_parameter("depth_normal", depth_normal)
    _update_sprite_scale(sprite_node, depth_normal, frame_delta)
    AnimationManager.update_perspective_tilt_mask(
        sprite_node.texture,
        character_body,
        sprite_node.global_position,
        sprite_node.scale,
        depth_normal,
        ascending
    )


func _update_sprite_scale(sprite: Sprite2D, depth_normal: float, _frame_delta: float) -> void:
    var scale_min: float = 0.5
    var scale_max: float = 1.0
    var smooth_depth: float = smoothstep(0.0, 1.0, depth_normal)
    sprite.scale = Vector2.ONE * lerp(scale_max, scale_min, smooth_depth)
