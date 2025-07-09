extends Node
class_name PitchDimension

var inner: PitchDimensionGodot

func _ready() -> void:
    inner = PitchDimensionGodot.new()
    add_child(inner)
    AudioPoolManager.play_music(inner.get_wav_stream())
