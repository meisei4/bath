extends Node2D
class_name GlacierFlow

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Glacier/glacier_main.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2


func _ready() -> void:
    #TODO: i just set the default for canvas items to this in the project settings but seriously its annoying
    self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    #iResolution = get_viewport_rect().size
    iResolution = Resolution.resolution
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
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
