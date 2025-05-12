extends Node2D
class_name Drekker

var SampleTexture: Image = Image.load_from_file("res://Assets/Textures/icebergs.jpg")

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Color/drekker_effect.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture


func _ready() -> void:
    self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    iChannel0 = ImageTexture.create_from_image(SampleTexture)
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
