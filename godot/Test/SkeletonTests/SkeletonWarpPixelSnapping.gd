extends Skeleton2D
class_name SkeletonWarpPixelSnapping

const TILE_SIZE: float = 8.0

func _ready() -> void:
    var shader: Shader = preload("res://Resources/Shaders/virtual_grid_snapping.gdshader")
    var shader_material: ShaderMaterial = ShaderMaterial.new()
    shader_material.shader = shader
    shader_material.set_shader_parameter("grid_size", TILE_SIZE)  # Pass TILE_SIZE for snapping
    self.material = shader_material
