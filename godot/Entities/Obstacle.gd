extends CharacterBody2D
class_name Obstacle

@export var path_speed: float = 0.0  # Speed along the path


func _ready() -> void:
    self._setup_signals()


func _setup_signals() -> void:
    pass
