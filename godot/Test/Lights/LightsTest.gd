extends Node2D
class_name LightsTest

var canvas_modulate: CanvasModulate
var point_light: PointLight2D


func _ready() -> void:
    _setup_canvas_modulate()
    _setup_point_light()


func _process(_delta: float) -> void:
    point_light.position = get_global_mouse_position()


func _setup_canvas_modulate() -> void:
    canvas_modulate = CanvasModulate.new()
    canvas_modulate.set_color(Color.GRAY)
    canvas_modulate.color.a = 1.0
    add_child(canvas_modulate)


func _setup_point_light() -> void:
    point_light = PointLight2D.new()
    point_light.color = Color.WHITE
    point_light.energy = 2.0
    point_light.set_texture_scale(0.75)
    point_light.shadow_filter_smooth = 4.0
    point_light.shadow_filter = PointLight2D.ShadowFilter.SHADOW_FILTER_PCF5
    point_light.shadow_enabled = true
    point_light.shadow_color = Color.BLACK  #TODO: this is funky, not sure how it works
    point_light.light_mask = 1
    #point_light.texture = preload(
        #"res://Assets/Lights/2d_lights_and_shadows_neutral_point_light.webp"
    #)
    point_light.texture = preload(
        "res://Assets/Lights/output_image.png"
    )
    add_child(point_light)
