extends Node2D
class_name TestScene

const MOVE_SPEED: int = 10

var camera: Camera2D
var width: int = 256
var height: int = 384
var scale_factor: int = 2  # Scale factor (e.g., 2 for 2x scaling) just to see the size better


func _ready() -> void:
	get_window().size = Vector2i(width * scale_factor, height * scale_factor)
	center_viewport()


func center_viewport() -> void:
	camera = Camera2D.new()  #TODO: COULD JUST ATTACH THE SNOW AND STUFF TO CAMERA
	camera.zoom = Vector2(1.0, 1.0)
	add_child(camera)


func _process(_delta: float) -> void:
	for child: Node in get_children():
		if child is Iruka:
			camera.position = child.position
