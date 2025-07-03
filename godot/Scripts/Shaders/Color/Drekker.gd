extends Node2D
class_name DrekkerColor

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = preload(ResourcePaths.DREKKER_EFFECT)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
#TODO: this is also resulting in texel level alpha/transparency issues in gl_compatability mode
# studying this as Forward+/Vulkan vs Compatibility/GL could perhaps also explain issues with the perspective tilt mask behavior
var iChannel0: Texture = preload(ResourcePaths.ICEBERGS_JPG)


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    BufferAShaderNode.owner = BufferA
    BufferA.owner = self
    MainImage.owner = self
