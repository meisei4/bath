extends Button
class_name AudioZoneTest

const MUSIC_TRACK_1: String = "res://Resources/Audio/Music/trimmed_10___What_You_Want_00-40-25_00-41-40.mp3"
const AUDIO_ZONE_SCENE_PATH: String = "res://godot/Test/AudioZoning/AutoGeneratedAudioZoneScene.tscn"

var audio_zone: AudioZone = null

func _ready() -> void:
    self.text = "AudioZone"
    var scene_creator = AudioZoneGen.new()
    scene_creator.create_and_save_audio_zone_scene()
    var audio_zone_scene: PackedScene = load(AUDIO_ZONE_SCENE_PATH) as PackedScene
    if not audio_zone_scene:
        push_error("Failed to load AudioZone scene from: " + AUDIO_ZONE_SCENE_PATH)
        return
    audio_zone = audio_zone_scene.instantiate() as AudioZone
    configure_audio_zone(audio_zone)
    get_parent().call_deferred("add_child", audio_zone)
    print("AudioZone added to parent node (deferred).")
    self.pressed.connect(_on_button_pressed)
    print("AudioZoneTest setup complete.")

func configure_audio_zone(audio_zone: AudioZone) -> void:
    audio_zone.global_position = self.global_position
    audio_zone.stream = preload(MUSIC_TRACK_1)
    audio_zone.max_effect_distance = 300.0
    audio_zone.effects_enabled = true
    audio_zone.effect_min_value = 0.0
    audio_zone.effect_max_value = 1.0
    print("AudioZone properties configured.")

func _on_button_pressed() -> void:
    if not audio_zone or not audio_zone.is_inside_tree():
        print("Error: AudioZone is not properly added to the scene tree.")
        return

    if audio_zone.playing:
        audio_zone.stop()
        print("AudioZone stopped.")
    else:
        if not audio_zone.stream:
            print("Error: No audio stream assigned to AudioZone.")
            return

        audio_zone.play()
        print("AudioZone started playing.")