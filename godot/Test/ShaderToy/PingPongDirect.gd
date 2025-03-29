extends Node2D
class_name PingPongDirect

var ShaderNode: ColorRect
var FeedbackShader: Shader = load("res://Resources/Shaders/simple_feedback_buffer.gdshader")
var FeedbackShaderMaterial: ShaderMaterial

var iMouse: Vector4

var BufferA: SubViewport
var BufferB: SubViewport
var ActiveBuffer: SubViewport
var InactiveBuffer: SubViewport
var FinalImage: TextureRect

func _ready() -> void:
    var main_viewport_size: Vector2 = get_viewport_rect().size
    BufferA = create_viewport(main_viewport_size)
    BufferB = create_viewport(main_viewport_size)
    ActiveBuffer = BufferA
    InactiveBuffer = BufferB

    FinalImage = TextureRect.new()
    FinalImage.size = main_viewport_size
    add_child(BufferA)
    add_child(BufferB)
    add_child(FinalImage)

    FeedbackShaderMaterial = ShaderMaterial.new()
    ShaderNode = ColorRect.new()
    ShaderNode.size = main_viewport_size
    FeedbackShaderMaterial.shader = FeedbackShader
    ShaderNode.material = FeedbackShaderMaterial

    ActiveBuffer.add_child(ShaderNode)
    FinalImage.texture = ActiveBuffer.get_texture()
    FeedbackShaderMaterial.set_shader_parameter("iResolution", main_viewport_size)
    FeedbackShaderMaterial.set_shader_parameter("iChannel0", FinalImage.get_texture())

func create_viewport(size: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = size
    subviewport.disable_3d = true
    subviewport.use_hdr_2d = true #TODO: THIS IS HUGE!
    RenderingServer.set_default_clear_color(Color(0.0, 0.0, 0.0, 1.0))
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport

#func _process(_delta: float) -> void:
    #var mouse_coords: Vector2 = get_viewport().get_mouse_position()
    #var mouse_z: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    #iMouse = Vector3(mouse_coords.x, mouse_coords.y, mouse_z)
    #FeedbackShaderMaterial.set_shader_parameter("iMouse", iMouse)
    #
    #FinalImage.texture = ActiveBuffer.get_texture()
    #ActiveBuffer.remove_child(ShaderNode)
    #InactiveBuffer.add_child(ShaderNode)
    #FeedbackShaderMaterial.set_shader_parameter("iChannel0", FinalImage.get_texture())
#
    #var tmp: SubViewport = ActiveBuffer
    #ActiveBuffer = InactiveBuffer
    #InactiveBuffer = tmp



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

    FeedbackShaderMaterial.set_shader_parameter("iMouse", iMouse)
    FinalImage.texture = ActiveBuffer.get_texture()
    ActiveBuffer.remove_child(ShaderNode)
    InactiveBuffer.add_child(ShaderNode)

    var tmp: SubViewport = ActiveBuffer
    ActiveBuffer = InactiveBuffer
    InactiveBuffer = tmp
