extends Node2D
class_name FeedbackBuffer

const BUFFER_SIZE: Vector2 = Vector2(512, 768)

var viewport_a: SubViewport
var viewport_b: SubViewport
var time_accum: float = 0.0
var current_target_is_a: bool = true


func _ready() -> void:
    viewport_a = SubViewport.new()
    viewport_a.size = BUFFER_SIZE
    add_child(viewport_a)

    viewport_b = SubViewport.new()
    viewport_b.size = BUFFER_SIZE
    add_child(viewport_b)

    var rect_a: ColorRect = ColorRect.new()
    rect_a.size = BUFFER_SIZE
    rect_a.material = _make_material()
    viewport_a.add_child(rect_a)

    var rect_b: ColorRect = ColorRect.new()
    rect_b.size = BUFFER_SIZE
    rect_b.material = _make_material()
    viewport_b.add_child(rect_b)


func _make_material() -> ShaderMaterial:
    var mat: ShaderMaterial = ShaderMaterial.new()
    mat.shader = load("res://Resources/Shaders/finite_approx_ripple.gdshader") as Shader
    return mat


func _process(delta: float) -> void:
    time_accum += delta
    var global_mouse: Vector2 = get_viewport().get_mouse_position()
    var mouse_pressed: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    var i_mouse: Vector3 = Vector3(global_mouse.x, global_mouse.y, mouse_pressed)

    set_uniforms(time_accum, i_mouse, BUFFER_SIZE)
    update_buffer()


func set_uniforms(i_time: float, i_mouse: Vector3, i_resolution: Vector2) -> void:
    var active_material: ShaderMaterial
    var feedback_texture: Texture

    if current_target_is_a:
        var rect_a: ColorRect = viewport_a.get_child(0) as ColorRect
        active_material = rect_a.material as ShaderMaterial
        feedback_texture = viewport_b.get_texture()
    else:
        var rect_b: ColorRect = viewport_b.get_child(0) as ColorRect
        active_material = rect_b.material as ShaderMaterial
        feedback_texture = viewport_a.get_texture()

    active_material.set_shader_parameter("iTime", i_time)
    active_material.set_shader_parameter("iMouse", i_mouse)
    active_material.set_shader_parameter("iResolution", i_resolution)
    active_material.set_shader_parameter("iChannel0", feedback_texture)


func update_buffer() -> void:
    current_target_is_a = not current_target_is_a


func get_output_texture() -> Texture:
    return viewport_b.get_texture() if current_target_is_a else viewport_a.get_texture()
