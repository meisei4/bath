extends Node2D
class_name WaterShaderTest

var shader_material: ShaderMaterial
var feedback_buffer: FeedbackBuffer
var time_accum: float = 0.0
const RESOLUTION = Vector2(512, 768)


func _ready() -> void:
    feedback_buffer = FeedbackBuffer.new()
    add_child(feedback_buffer)
    await get_tree().process_frame
    var shader = load("res://Resources/Shaders/water.gdshader")
    shader_material = ShaderMaterial.new()
    shader_material.shader = shader
    shader_material.set_shader_parameter(
        "iChannel0", load("res://Assets/Textures/gray_noise_small.png")
    )
    shader_material.set_shader_parameter("iChannel1", load("res://Assets/Textures/rocks.jpg"))
    shader_material.set_shader_parameter("iChannel2", load("res://Assets/Textures/pebbles.png"))
    shader_material.set_shader_parameter("iResolution", RESOLUTION)
    var rect = ColorRect.new()
    rect.material = shader_material
    rect.color = Color.WHITE
    rect.size = RESOLUTION
    add_child(rect)


func _process(delta: float) -> void:
    time_accum += delta
    var global_mouse: Vector2 = get_viewport().get_mouse_position()
    var mouse_pressed: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    var i_mouse: Vector3 = Vector3(global_mouse.x, global_mouse.y, mouse_pressed)
    shader_material.set_shader_parameter("iTime", time_accum)
    shader_material.set_shader_parameter("iMouse", i_mouse)
    shader_material.set_shader_parameter("iChannel3", feedback_buffer.get_output_texture())
    feedback_buffer.set_uniforms(time_accum, i_mouse, RESOLUTION)
    feedback_buffer.update_buffer()
