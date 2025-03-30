extends Node2D
class_name Debug

var DebugShaderNode: ColorRect
var DebugShader: Shader = load("res://Resources/Shaders/fragcoord_inversion_debug.gdshader")
var DebugShaderMaterial: ShaderMaterial

var PropagationShaderNode: ColorRect
var PropagationShader: Shader = load("res://Resources/Shaders/fragcoord_invert_propagation_test.gdshader")
var PropagationShaderMaterial: ShaderMaterial

var iResolution: Vector2
var iMouse: Vector3
var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

func _ready() -> void:
    iResolution = get_viewport_rect().size

    BufferA = create_viewport(iResolution)
    BufferA.use_hdr_2d = true # buffer sampling becomes float 16-bit
    DebugShaderMaterial = ShaderMaterial.new()
    DebugShaderNode = ColorRect.new()
    DebugShaderNode.size = iResolution
    DebugShaderMaterial.shader = DebugShader
    DebugShaderNode.material = DebugShaderMaterial
    DebugShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferA.add_child(DebugShaderNode)
    add_child(BufferA)

    BufferB = create_viewport(iResolution)
    BufferB.use_hdr_2d = true # buffer sampling becomes float 16-bit
    PropagationShaderMaterial = ShaderMaterial.new()
    PropagationShaderNode = ColorRect.new()
    PropagationShaderNode.size = iResolution
    PropagationShaderMaterial.shader =  PropagationShader
    PropagationShaderNode.material = PropagationShaderMaterial
    PropagationShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferB.add_child(PropagationShaderNode)
    add_child(BufferB)

    MainImage = TextureRect.new()
    MainImage.texture = BufferB.get_texture()
    add_child(MainImage)


func create_viewport(size: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = size
    subviewport.disable_3d = true
    RenderingServer.set_default_clear_color(Color.BLACK)
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport


func _process(delta: float) -> void:
    var mouse_xy: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    #iMouse = Vector3(mouse_xy.x, mouse_xy.y, mouse_z)
    iMouse = Vector3(mouse_xy.x, iResolution.y - mouse_xy.y, mouse_z);
    DebugShaderMaterial.set_shader_parameter("iMouse", iMouse)
    PropagationShaderMaterial.set_shader_parameter("iChannel0", BufferA.get_texture())
