extends Node
class_name DiveAnimation

var shader: Shader = preload(ResourcePaths.DIVE_SHADER)


func process_animation(
    current_depth_position: float,
    depth_normal: float,
    ascending: bool,
    frame_delta: float,
    sprite: Sprite2D
) -> void:
    if sprite.material == null:
        sprite.material = ShaderMaterial.new()
    if sprite.material.shader != self.shader:
        sprite.material.shader = self.shader
    var sprite_shader_material: ShaderMaterial = sprite.material
    var vertical_offset_pixels: float = SpacetimeManager.to_physical_space(current_depth_position)
    sprite.position.y = -vertical_offset_pixels
    sprite_shader_material.set_shader_parameter("iChannel0", sprite.texture)
    sprite_shader_material.set_shader_parameter("ascending", ascending)
    sprite_shader_material.set_shader_parameter("depth_normal", depth_normal)
    _update_sprite_scale(sprite, depth_normal, frame_delta)


func _update_sprite_scale(sprite: Sprite2D, depth_normal: float, _frame_delta: float) -> void:
    var scale_min: float = 0.5
    var scale_max: float = 1.0
    var smooth_depth: float = smoothstep(0.0, 1.0, depth_normal)
    sprite.scale = Vector2.ONE * lerp(scale_max, scale_min, smooth_depth)
