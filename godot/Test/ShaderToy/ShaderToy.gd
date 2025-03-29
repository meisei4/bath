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

var BackgroundTexture: Texture = load("res://Assets/Textures/rocks.jpg")
var CausticsTexture: Texture = load("res://Assets/Textures/pebbles.png")


var iMouse: Vector4
var BufferA: SubViewport
var BufferB: SubViewport
var BufferC: SubViewport
var ActiveBuffer: SubViewport
var InactiveBuffer: SubViewport

var RippleImage: TextureRect
var FinalImage: TextureRect

func _ready() -> void:
    var main_viewport_size: Vector2 = get_viewport_rect().size
    BufferA = create_viewport(main_viewport_size)
    BufferB = create_viewport(main_viewport_size)
    ActiveBuffer = BufferA
    InactiveBuffer = BufferB

    RippleImage = TextureRect.new()

    RippleImage.size = main_viewport_size
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
    FinalImage = TextureRect.new()
    FinalImage.size = main_viewport_size
    add_child(BufferC)
    add_child(FinalImage)

    WaterShaderMaterial = ShaderMaterial.new()
    WaterShaderNode = ColorRect.new()
    WaterShaderNode.size = main_viewport_size
    WaterShaderMaterial.shader = WaterShader
    WaterShaderNode.material = WaterShaderMaterial

    BufferC.add_child(WaterShaderNode)

    FinalImage.texture = BufferC.get_texture()
    WaterShaderMaterial.set_shader_parameter("iResolution", main_viewport_size)
    WaterShaderMaterial.set_shader_parameter("iChannel0", RippleImage.get_texture())
    #WaterShaderMaterial.set_shader_parameter("iChannel0", CausticsTexture) #TODO: shadertoy does wrapping = repeat not clamp see
    WaterShaderMaterial.set_shader_parameter("iChannel1", BackgroundTexture)


func create_viewport(size: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = size
    subviewport.disable_3d = true
    #TODO: this was the fix! it allows for the texture format for the subviewport sampling to go from R10G10B10A2_UNORM (10 bit precision unsigned normalized) to 16 bit FLOATS!
    subviewport.use_hdr_2d = true
    RenderingServer.set_default_clear_color(Color(0.0, 0.0, 0.0, 0.0))
    subviewport.transparent_bg
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport

var mouse_pressed: bool = false
var drag_start: Vector2 = Vector2()

func _process(_delta: float) -> void:
    var current_pos: Vector2 = get_viewport().get_mouse_position()
    if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
        if not mouse_pressed:
            drag_start = current_pos
            iMouse = Vector4(current_pos.x, current_pos.y, drag_start.x, drag_start.y)
            mouse_pressed = true
        else:
            iMouse.x = current_pos.x
            iMouse.y = current_pos.y
    else:
        if mouse_pressed:
            iMouse = Vector4(current_pos.x, current_pos.y, -drag_start.x, -drag_start.y)
            mouse_pressed = false

    RippleShaderMaterial.set_shader_parameter("iMouse", iMouse)
    RippleImage.texture = ActiveBuffer.get_texture()
    ActiveBuffer.remove_child(RippleShaderNode)
    InactiveBuffer.add_child(RippleShaderNode)

    var tmp: SubViewport = ActiveBuffer
    ActiveBuffer = InactiveBuffer
    InactiveBuffer = tmp
