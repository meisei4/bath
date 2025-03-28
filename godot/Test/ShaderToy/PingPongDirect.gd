extends Node2D
class_name PingPongDirect

var shader: Shader = load("res://Resources/Shaders/simple_feedback_buffer.gdshader")
var shader_material: ShaderMaterial

var iTime: float = 0.0
var iMouse: Vector3 = Vector3(0.0, 0.0, 0.0)

var buffer_A: SubViewport
var buffer_B: SubViewport
var active_buffer: SubViewport
var inactive_buffer: SubViewport
var shader_node: CanvasItem
var image: CanvasItem

func _ready() -> void:
    var main_viewport_size: Vector2 = get_viewport_rect().size
    buffer_A = create_viewport(main_viewport_size)
    buffer_B = create_viewport(main_viewport_size)
    active_buffer = buffer_A
    inactive_buffer = buffer_B

    image = TextureRect.new()
    image.size = main_viewport_size
    add_child(buffer_A)
    add_child(buffer_B)
    add_child(image)

    shader_material = ShaderMaterial.new()
    shader_node = ColorRect.new() #only this works for shader
    shader_node.size = main_viewport_size
    shader_material.shader = shader
    shader_node.material = shader_material

    shader_material.set_shader_parameter("iResolution", main_viewport_size)

    active_buffer.add_child(shader_node)
    await RenderingServer.frame_post_draw
    image.texture = active_buffer.get_texture()
    shader_material.set_shader_parameter("iChannel0", image.get_texture())
    #Control
    shader_node.anchor_left = 0.0
    shader_node.anchor_top = 0.0
    shader_node.anchor_right = 0.0
    shader_node.anchor_bottom = 0.0
    #shader_node.offset_left = 0.0
    #shader_node.offset_top = 0.0
    #shader_node.offset_right = 0.0
    #shader_node.offset_bottom = 0.0
    shader_node.pivot_offset = Vector2(0, 0)
    shader_node.position = Vector2(0, 0)
    shader_node.rotation = 0.0
    shader_node.rotation_degrees = 0.0
    shader_node.scale = Vector2(1, 1)

    shader_node.focus_mode = Control.FOCUS_NONE
    shader_node.grow_horizontal = Control.GROW_DIRECTION_END
    shader_node.grow_vertical = Control.GROW_DIRECTION_END
    shader_node.layout_direction = Control.LAYOUT_DIRECTION_INHERITED


func create_viewport(size: Vector2) -> SubViewport:
    var vp = SubViewport.new()
    vp.size = size
    vp.disable_3d = true
    vp.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS

    return vp


func _process(delta: float) -> void:
    var mouse_coords: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector3(mouse_coords.x, mouse_coords.y, mouse_z)
    shader_material.set_shader_parameter("iMouse", iMouse)

    image.texture = active_buffer.get_texture()
    active_buffer.remove_child(shader_node)
    inactive_buffer.add_child(shader_node)
    shader_material.set_shader_parameter("iChannel0", image.get_texture())

    var temp = active_buffer
    active_buffer = inactive_buffer
    inactive_buffer = temp
