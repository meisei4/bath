extends Node2D
class_name PerspectiveTiltMaskFragment

const MAXIMUM_SPRITE_COUNT: int = 12

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load(
    "res://Resources/Shaders/MechanicAnimations/perspective_tilt_mask.gdshader"
)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2

var _sprite_textures: Array[Texture2D] = []
var _sprite_data0: PackedVector4Array
var _sprite_data1: PackedVector4Array


func _ready() -> void:
    _sprite_textures.resize(MAXIMUM_SPRITE_COUNT)
    _sprite_data0.resize(MAXIMUM_SPRITE_COUNT)
    _sprite_data1.resize(MAXIMUM_SPRITE_COUNT)
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferA.render_target_clear_mode = SubViewport.CLEAR_MODE_ALWAYS
    BufferA.transparent_bg = true
    BufferA.use_hdr_2d = false
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    #BufferAShaderMaterial.set_shader_parameter("sprite_count", 0)
    #BufferAShaderMaterial.set_shader_parameter("sprite_textures", _sprite_textures)
    #BufferAShaderMaterial.set_shader_parameter("sprite_data0", _sprite_data0)
    #BufferAShaderMaterial.set_shader_parameter("sprite_data1", _sprite_data1)
    BufferAShaderMaterial.set_shader_parameter("sprite_texture", _sprite_textures[0])
    BufferAShaderMaterial.set_shader_parameter("sprite_data0", _sprite_data0[0])
    BufferAShaderMaterial.set_shader_parameter("sprite_data1", _sprite_data1[0])
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    add_child(MainImage)
    ComputeShaderSignalManager.register_perspective_tilt_mask_fragment(self)


func set_sprite_data(
    sprite_index: int,
    center_px: Vector2,
    half_size_px: Vector2,
    altitude_normal: float,
    ascending: bool
) -> void:
    if sprite_index < 0 or sprite_index >= MAXIMUM_SPRITE_COUNT:
        return
    _sprite_data0[sprite_index] = Vector4(center_px.x, center_px.y, half_size_px.x, half_size_px.y)
    _sprite_data1[sprite_index] = Vector4(altitude_normal, 1.0 if ascending else 0.0, 0.0, 0.0)
    BufferAShaderMaterial.set_shader_parameter("sprite_data0", _sprite_data0)
    BufferAShaderMaterial.set_shader_parameter("sprite_data1", _sprite_data1)


func set_sprite_texture(sprite_index: int, tex: Texture2D) -> void:
    if sprite_index < 0 or sprite_index >= MAXIMUM_SPRITE_COUNT:
        return
    _sprite_textures[sprite_index] = tex
    BufferAShaderMaterial.set_shader_parameter("sprite_textures", _sprite_textures)


func get_perspective_tilt_mask_texture_fragment() -> Texture:
    return BufferA.get_texture()


func register_sprite_texture(sprite_texture: Texture2D) -> int:
    var index = _sprite_textures.find(null)
    if index == -1:
        push_error("No more mask slots!")
        return -1
    _sprite_textures[index] = sprite_texture
    BufferAShaderMaterial.set_shader_parameter("sprite_textures", _sprite_textures)
    BufferAShaderMaterial.set_shader_parameter("sprite_count", index + 1)
    return index
