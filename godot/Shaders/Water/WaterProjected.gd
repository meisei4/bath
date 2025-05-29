extends Node2D
class_name WaterProjected

var RippleShaderNode: ColorRect
var RippleShader: Shader = preload("res://Resources/Shaders/Water/finite_approx_ripple.gdshader")
var RippleShaderMaterial: ShaderMaterial

var WaterShaderNode: ColorRect
var WaterShader: Shader = preload("res://Resources/Shaders/Water/water_projected.gdshader")
var WaterShaderMaterial: ShaderMaterial

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2

var iChannel0: Texture = preload("res://Assets/Textures/gray_noise_small.png")
var iChannel1: Texture = preload("res://Assets/Textures/moon_water.png")
var iChannel2: Texture = preload("res://Assets/Textures/pebbles.png")
var iChannel3: Texture


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferA.use_hdr_2d = true
    RippleShaderMaterial = ShaderMaterial.new()
    RippleShaderNode = ColorRect.new()
    RippleShaderNode.size = iResolution
    RippleShaderMaterial.shader = RippleShader
    RippleShaderNode.material = RippleShaderMaterial
    RippleShaderMaterial.set_shader_parameter("iResolution", iResolution)
    RippleShaderMaterial.set_shader_parameter("tile_size", GlacierConstants.TILE_SIZE_1D)

    BufferB = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferB.use_hdr_2d = false
    WaterShaderMaterial = ShaderMaterial.new()
    WaterShaderNode = ColorRect.new()
    WaterShaderNode.size = iResolution
    WaterShaderMaterial.shader = WaterShader
    WaterShaderNode.material = WaterShaderMaterial
    WaterShaderMaterial.set_shader_parameter("iResolution", iResolution)
    WaterShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    WaterShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    WaterShaderMaterial.set_shader_parameter("iChannel2", iChannel2)

    MainImage = TextureRect.new()
    MainImage.texture = BufferB.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(RippleShaderNode)
    add_child(BufferA)
    BufferB.add_child(WaterShaderNode)
    add_child(BufferB)
    add_child(MainImage)


func _process(_delta: float) -> void:
    iChannel3 = BufferA.get_texture() as ViewportTexture
    WaterShaderMaterial.set_shader_parameter("iChannel3", iChannel3)


func get_water_texture() -> Texture:
    return BufferB.get_texture()
