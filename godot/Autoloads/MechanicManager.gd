extends Node

signal left_lateral_movement
signal right_lateral_movement
signal jump


func _process(_delta: float) -> void:
    if Input.is_action_pressed("left"):
        left_lateral_movement.emit()
    if Input.is_action_pressed("right"):
        right_lateral_movement.emit()


#TODO: this is just for testing the difference between InputMap (per frame polling) vs InputEvent direct signal handling access
func _unhandled_input(event: InputEvent) -> void:
    if event is InputEventKey and event.pressed and event.keycode == Key.KEY_SPACE:
        jump.emit()
