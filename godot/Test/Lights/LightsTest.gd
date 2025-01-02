extends Node2D
class_name LightsTest

var canvas_modulate: CanvasModulate
var point_light: PointLight2D
var light_occluders: Array = []


func _ready() -> void:
    _setup_canvas_modulate()
    _setup_point_light()
    #_setup_light_occluders_test()


func _process(_delta: float) -> void:
    point_light.position = get_global_mouse_position()


func _setup_canvas_modulate() -> void:
    canvas_modulate = CanvasModulate.new()
    canvas_modulate.color = Color.DEEP_PINK
    canvas_modulate.color.a = 1.0
    add_child(canvas_modulate)


func _setup_point_light() -> void:
    point_light = PointLight2D.new()
    point_light.color = Color.WHITE
    point_light.energy = 2.0
    point_light.shadow_enabled = true
    point_light.shadow_color = Color.BLACK  #TODO: this is funky, not sure how it works
    point_light.light_mask = 1
    point_light.texture = preload(
        "res://Assets/Lights/2d_lights_and_shadows_neutral_point_light.webp"
    )
    add_child(point_light)


func _setup_light_occluders_test() -> void:
    var light_occluder: LightOccluder2D = LightOccluder2D.new()
    var occluder: OccluderPolygon2D = OccluderPolygon2D.new()
    occluder.polygon = PackedVector2Array(
        [Vector2(-50, -50), Vector2(50, -50), Vector2(50, 50), Vector2(-50, 50)]
    )
    light_occluder.occluder_light_mask = 1
    light_occluder.occluder = occluder
    add_child(light_occluder)
    light_occluders.append(light_occluder)
