extends Node2D
class_name GlacierFlow

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Glacier/glacier_main.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2

func _ready() -> void:
    #var res: Vector2i = Vector2i(855, 480)
    #DisplayServer.window_set_size(res)  #TODO: this doesnt do what you think it does
    #iResolution = res
    iResolution = get_viewport_rect().size
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
