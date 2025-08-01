extends Node2D
class_name PerspectiveTiltMaskFragment

const MAXIMUM_SPRITE_COUNT: int = 12

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = preload(ResourcePaths.PERSPECTIVE_TILT_MASK_SHADER)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2

#TODO: YOU ARE DOING OPTIMISM STOP!
#you only have one sprite to test why would you try to account for more????!!!
# also it wont work in opengl anywyas because too many samplers in the shader:

# :E 0:00:01:279   _display_error_with_code: CanvasShaderGLES3: Program linking failed:
# WARNING: Output of vertex shader 'varying_G' not read by fragment shader
# ERROR: Implementation limit of 16 active fragment shader samplers (e.g., maximum number of supported image units) exceeded, fragment shader uses 19 samplers
#<C++ Source>  drivers/gles3/shader_gles3.cpp:265 @ _display_error_with_code()

var _sprite_textures: Array[Texture2D] = []
var sprite_position_data: PackedVector2Array
var altitude_normal_data: PackedFloat32Array
var ascending_data: PackedInt32Array


func _ready() -> void:
    _sprite_textures.resize(MAXIMUM_SPRITE_COUNT)
    sprite_position_data.resize(MAXIMUM_SPRITE_COUNT)
    altitude_normal_data.resize(MAXIMUM_SPRITE_COUNT)
    ascending_data.resize(MAXIMUM_SPRITE_COUNT)
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferA.render_target_clear_mode = SubViewport.CLEAR_MODE_ALWAYS
    BufferA.transparent_bg = true
    BufferA.use_hdr_2d = false
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderMaterial.shader = BufferAShader

    BufferAShaderMaterial.set_shader_parameter("iChannel0", _sprite_textures[0])
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)

    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderNode.material = BufferAShaderMaterial
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    BufferAShaderNode.owner = BufferA
    BufferA.owner = self
    MainImage.owner = self
    MaskManager.register_perspective_tilt_mask_fragment(self)


func set_sprite_data(
    sprite: Sprite2D, sprite_index: int, altitude_normal: float, ascending: bool
) -> void:
    if sprite_index < 0 or sprite_index >= MAXIMUM_SPRITE_COUNT:
        return
    sprite_position_data[sprite_index] = sprite.global_position
    altitude_normal_data[sprite_index] = altitude_normal
    ascending_data[sprite_index] = 1 if ascending else 0
    _sprite_textures[sprite_index] = sprite.texture
    BufferAShaderMaterial.set_shader_parameter("iChannel0", _sprite_textures[0])
    BufferAShaderMaterial.set_shader_parameter(
        "sprite_texture_size", _sprite_textures[0].get_size()
    )
    BufferAShaderMaterial.set_shader_parameter("sprite_scale", sprite.scale)
    BufferAShaderMaterial.set_shader_parameter("sprite_position", sprite_position_data[0])
    BufferAShaderMaterial.set_shader_parameter("altitude_normal", altitude_normal_data[0])
    BufferAShaderMaterial.set_shader_parameter("ascending", ascending_data[0])


func set_sprite_rotation(sprite: Sprite2D, sprite_index: int) -> void:
    if sprite_index < 0 or sprite_index >= MAXIMUM_SPRITE_COUNT:
        return
    pass


func get_perspective_tilt_mask_texture_fragment() -> Texture:
    return BufferA.get_texture()


func register_sprite_texture(sprite_texture: Texture2D) -> int:
    var index: int = _sprite_textures.find(null)
    if index == -1:
        push_error("No more mask slots!")
        return -1
    _sprite_textures[index] = sprite_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel0", _sprite_textures[0])
    return index
