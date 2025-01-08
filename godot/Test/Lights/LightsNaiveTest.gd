extends Node2D
class_name LightsNaiveTest

var occluder_polygon: Array[Vector2] = [
    Vector2.ZERO,
    Vector2.ZERO,
    Vector2.ZERO,
    Vector2.ZERO
]

var canvas_modulate: CanvasModulate
var naive_light_material: ShaderMaterial
var debug_drawer: OccluderDebugDrawer

var light_position: Vector2 = Vector2.ZERO
var light_radius: float = 80.0
var light_color: Color = Color.WHITE
var shadow_color: Color = Color.BLACK

func _ready() -> void:
    _setup_canvas_modulate()
    naive_light_material = ShaderMaterial.new()
    naive_light_material.shader = preload("res://Resources/Shaders/light_shader.gdshader")
    canvas_modulate.material = naive_light_material

    debug_drawer = OccluderDebugDrawer.new()
    add_child(debug_drawer)

    var base_pos: Vector2 = Vector2(0, 0)
    var box_size: Vector2 = Vector2(100, 100)
    occluder_polygon[0] = base_pos
    occluder_polygon[1] = base_pos + Vector2(box_size.x, 0.0)
    occluder_polygon[2] = base_pos + box_size
    occluder_polygon[3] = base_pos + Vector2(0.0, box_size.y)
    debug_drawer.occluder_polygon = occluder_polygon
    _update_shader_params()

func _setup_canvas_modulate() -> void:
    canvas_modulate = CanvasModulate.new()
    canvas_modulate.set_color(Color(0.5, 0.5, 0.5, 1.0))
    canvas_modulate.color.a = 1.0
    add_child(canvas_modulate)

func _process(delta: float) -> void:
    var viewport: Window = get_viewport() as Window
    light_position = viewport.get_mouse_position()
    naive_light_material.set_shader_parameter("u_light_pos", light_position)
    print("Light pos:", light_position)
    print("Occluder:", occluder_polygon)

func _update_shader_params() -> void:
    var occluder_floats: PackedFloat32Array = PackedFloat32Array()
    var viewport: Window = get_viewport() as Window
    for vertex: Vector2 in occluder_polygon:
        occluder_floats.append(vertex.x)
        occluder_floats.append(vertex.y)
    naive_light_material.set_shader_parameter("u_occluder", occluder_floats)
    naive_light_material.set_shader_parameter("u_light_radius", light_radius)
    naive_light_material.set_shader_parameter("u_light_color", light_color)
    naive_light_material.set_shader_parameter("u_shadow_color", shadow_color)
    naive_light_material.set_shader_parameter("u_screen_size", viewport.size)
