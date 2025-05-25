extends Node2D
class_name WaterProjected

var RippleShaderNode: ColorRect
var RippleShader: Shader = load("res://Resources/Shaders/Water/finite_approx_ripple.gdshader")
var RippleShaderMaterial: ShaderMaterial

var WaterShaderNode: ColorRect
var WaterShader: Shader = load("res://Resources/Shaders/Water/water_projected.gdshader")
var WaterShaderMaterial: ShaderMaterial

var noise_texture_resource: Texture2D = (
    preload("res://Assets/Textures/gray_noise_small.png") as Texture2D
)
var NoiseTexture: Image = noise_texture_resource.get_image()

var background_texture_resource: Texture2D = (
    preload("res://Assets/Textures/moon_water.png") as Texture2D
)
var BackgroundTexture: Image = background_texture_resource.get_image()

var caustics_texture_resource: Texture2D = preload("res://Assets/Textures/pebbles.png") as Texture2D
var CausticsTexture: Image = caustics_texture_resource.get_image()

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2

var iChannel0: Texture
#TODO: replace background texture with glacier waters depth texture and proejction alignment/zoom fix
var iChannel1: Texture
var iChannel2: Texture
var iChannel3: Texture


func _ready() -> void:
    initialize_shadertoy_uniforms_and_textures()
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


func initialize_shadertoy_uniforms_and_textures() -> void:
    iResolution = ResolutionManager.resolution
    #NoiseTexture.convert(Image.FORMAT_R8)
    #BackgroundTexture.convert(Image.FORMAT_RGBA8)
    #CausticsTexture.convert(Image.FORMAT_R8)
    iChannel0 = ImageTexture.create_from_image(NoiseTexture)
    iChannel1 = ImageTexture.create_from_image(BackgroundTexture)
    iChannel2 = ImageTexture.create_from_image(CausticsTexture)


func _process(delta: float) -> void:
    iChannel3 = BufferA.get_texture() as ViewportTexture
    WaterShaderMaterial.set_shader_parameter("iChannel3", iChannel3)


func get_water_texture() -> Texture:
    return BufferB.get_texture()
