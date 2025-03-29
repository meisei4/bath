#TODO: figure out the reason for the buffer errors again. for some reason they are being not ping ponged in time
extends Node2D
class_name ShaderToy

var WaterShaderNode: ColorRect
var WaterShader: Shader = load("res://Resources/Shaders/water.gdshader")
#var WaterShader: Shader = load("res://Resources/Shaders/buffer_sampling_clamp_test_main.gdshader")
var WaterShaderMaterial: ShaderMaterial

var RippleShaderNode: ColorRect
var RippleShader: Shader = load("res://Resources/Shaders/finite_approx_ripple.gdshader")
#var RippleShader: Shader = load("res://Resources/Shaders/buffer_sampling_clamp_test.gdshader")
var RippleShaderMaterial: ShaderMaterial

var NoiseTexture: ImageTexture
var NoiseImage: Image = Image.load_from_file("res://Assets/Textures/gray_noise_small.png")
var BackgroundTexture: ImageTexture
var BackgroundImage: Image = Image.load_from_file("res://Assets/Textures/rocks.jpg")
var CausticsTexture: ImageTexture
var CausticsImage: Image = Image.load_from_file("res://Assets/Textures/pebbles.png")

#var iMouse: Vector3
var iMouse: Vector4
var iTime: float
var BufferA: SubViewport
var BufferB: SubViewport
var BufferC: SubViewport
var ActiveBuffer: SubViewport
var InactiveBuffer: SubViewport

var RippleImage: TextureRect
var FinalImage: TextureRect


func _ready() -> void:
    NoiseImage.convert(Image.FORMAT_R8)
    NoiseTexture = ImageTexture.create_from_image(NoiseImage)
    BackgroundImage.convert(Image.FORMAT_RGBA8)
    BackgroundTexture = ImageTexture.create_from_image(BackgroundImage)
    CausticsImage.convert(Image.FORMAT_R8)
    CausticsTexture = ImageTexture.create_from_image(CausticsImage)

    var main_viewport_size: Vector2 = get_viewport_rect().size
    BufferA = create_viewport(main_viewport_size)
    BufferB = create_viewport(main_viewport_size)
    #TODO: this was the fix! it allows for the texture format for the subviewport sampling to go from R10G10B10A2_UNORM (10 bit precision unsigned normalized) to 16 bit FLOATS!
    BufferA.use_hdr_2d = true
    BufferB.use_hdr_2d = true
    ActiveBuffer = BufferA
    InactiveBuffer = BufferB

    RippleImage = TextureRect.new()
    RippleImage.size = main_viewport_size
    #RippleImage.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
    #RippleImage.texture_repeat = CanvasItem.TEXTURE_REPEAT_DISABLED
    #RippleImage.flip_v = true
    add_child(BufferA)
    add_child(BufferB)
    add_child(RippleImage)

    RippleShaderMaterial = ShaderMaterial.new()
    RippleShaderNode = ColorRect.new()
    RippleShaderNode.size = main_viewport_size
    RippleShaderMaterial.shader = RippleShader
    RippleShaderNode.material = RippleShaderMaterial

    ActiveBuffer.add_child(RippleShaderNode)
    RippleImage.texture = ActiveBuffer.get_texture()
    RippleShaderMaterial.set_shader_parameter("iResolution", main_viewport_size)
    RippleShaderMaterial.set_shader_parameter("iChannel0", RippleImage.get_texture())

    BufferC = create_viewport(main_viewport_size)
    BufferC.use_hdr_2d = false  #TODO: without this the noise texture goes insane
    FinalImage = TextureRect.new()
    FinalImage.size = main_viewport_size
    FinalImage.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
    FinalImage.texture_repeat = CanvasItem.TEXTURE_REPEAT_ENABLED
    #FinalImage.flip_v = true
    add_child(BufferC)
    add_child(FinalImage)

    WaterShaderMaterial = ShaderMaterial.new()
    WaterShaderNode = ColorRect.new()
    WaterShaderNode.size = main_viewport_size
    WaterShaderMaterial.shader = WaterShader
    WaterShaderNode.material = WaterShaderMaterial

    BufferC.add_child(WaterShaderNode)
    await RenderingServer.frame_post_draw
    FinalImage.texture = BufferC.get_texture()
    WaterShaderMaterial.set_shader_parameter("iResolution", main_viewport_size)
    WaterShaderMaterial.set_shader_parameter("iChannel0", NoiseTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel1", BackgroundTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel2", CausticsTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel3", RippleImage.get_texture())


func create_viewport(size: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = size
    subviewport.disable_3d = true
    RenderingServer.set_default_clear_color(Color.BLACK)
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport


var mouse_pressed: bool = false
var drag_start: Vector2 = Vector2()


func _process(delta: float) -> void:
    iTime += delta
    WaterShaderMaterial.set_shader_parameter("iTime", iTime)

    var mouse_xy: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector4(mouse_xy.x, mouse_xy.y, mouse_z, 0.0)

    RippleShaderMaterial.set_shader_parameter("iMouse", iMouse)
    RippleImage.texture = ActiveBuffer.get_texture()
    RippleShaderMaterial.set_shader_parameter("iChannel0", RippleImage.get_texture())
    WaterShaderMaterial.set_shader_parameter("iChannel3", RippleImage.get_texture())
    WaterShaderMaterial.set_shader_parameter("iChannel0", NoiseTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel1", BackgroundTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel2", CausticsTexture)

    ActiveBuffer.remove_child(RippleShaderNode)
    InactiveBuffer.add_child(RippleShaderNode)

    var tmp: SubViewport = ActiveBuffer
    ActiveBuffer = InactiveBuffer
    InactiveBuffer = tmp
