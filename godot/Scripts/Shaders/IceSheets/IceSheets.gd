extends Node2D
class_name IceSheets

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = preload(ResourcePaths.ICE_SHEETS_SHADER)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport

var MainImage: TextureRect
var AlphaOverrideShader: Shader = preload(ResourcePaths.FREE_ALPHA_CHANNEL)
var MainImageMaterial: ShaderMaterial

var iResolution: Vector2

var iChannel0: Texture


func _ready() -> void:
    iResolution = ResolutionManager.resolution

    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    MainImage.z_index = -1  #TODO: gross
    MainImageMaterial = ShaderMaterial.new()
    MainImageMaterial.shader = AlphaOverrideShader
    MainImageMaterial.set_shader_parameter("iChannel0", BufferA.get_texture())

    MainImage.material = MainImageMaterial
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    MaskManager.register_ice_sheets(self)


func _process(delta: float) -> void:
    MaskManager.iTime += delta
    BufferAShaderMaterial.set_shader_parameter("iTime", MaskManager.iTime)
