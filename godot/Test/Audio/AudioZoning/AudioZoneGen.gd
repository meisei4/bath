extends Node
class_name AudioZoneGen

const AUDIO_ZONE_SCENE_PATH: String = "res://godot/Test/Audio/AudioZoning/AudioZone.tscn"


func _ready() -> void:
    create_and_save_audio_zone_scene()


func create_and_save_audio_zone_scene() -> void:
    if ResourceLoader.exists(AUDIO_ZONE_SCENE_PATH):
        print("AudioZone scene already exists at: ", AUDIO_ZONE_SCENE_PATH)
        return
    var audio_zone: AudioZone = AudioZone.new()
    var packed_scene: PackedScene = PackedScene.new()
    packed_scene.pack(audio_zone)
    ResourceSaver.save(packed_scene, AUDIO_ZONE_SCENE_PATH)
