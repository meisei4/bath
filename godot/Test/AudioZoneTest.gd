extends Button
class_name AudioZoneTest

var audio_zone: AudioZone

const MUSIC_TRACK_1: String = "res://Resources/Audio/Music/trimmed_10___What_You_Want_00-40-25_00-41-40.mp3"


func _ready() -> void:
    self.text = "AudioZone"
    audio_zone = AudioZone.new()
    configure_audio_zone(audio_zone)
    get_parent().add_child(audio_zone)
    self.pressed.connect(_on_button_pressed)


func configure_audio_zone(audio_zone: AudioZone) -> void:
    audio_zone.global_position = self.global_position
    audio_zone.stream = preload(MUSIC_TRACK_1)
    audio_zone.max_effect_distance = 300.0
    audio_zone.effects_enabled = true
    audio_zone.effect_min_value = 0.0
    audio_zone.effect_max_value = 1.0


func _on_button_pressed() -> void:
    if audio_zone.playing:
        audio_zone.stop()
    else:
        audio_zone.play()
