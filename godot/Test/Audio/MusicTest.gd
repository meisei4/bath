extends Control
class_name MusicTest

var current_pitch: float = 1.0
const PITCH_STEP: float = 0.1
const MAX_PITCH: float = 2.0
const MIN_PITCH: float = 0.5

const MUSIC_TRACK_1: String = "res://Resources/Audio/Music/trimmed_10___What_You_Want_00-40-25_00-41-40.mp3"
const MUSIC_TRACK_2: String = "res://Resources/Audio/Music/trimmed_01___Only_Shallow_00-03-40_00-04-17.mp3"
const MUSIC_TRACK_3: String = "res://Resources/Audio/Music/trimmed_03___Touched_00-06-55_00-07-52.mp3"
const MUSIC_TRACK_4: String = "res://Resources/Audio/Music/trimmed_04___To_Here_Knows_When_00-12-38_00-13-23.mp3"
const MUSIC_TRACK_5: String = "res://Resources/Audio/Music/trimmed_05___When_You_Sleep_00-17-20_00-17-35.mp3"

var music_list: Array[String] = [
    MUSIC_TRACK_1,
    MUSIC_TRACK_2,
    MUSIC_TRACK_3,
    MUSIC_TRACK_4,
    MUSIC_TRACK_5,
]

var option_button_music: OptionButton
var button_play_music: Button
var button_stop_all_music: Button
var button_enable_reverb_music: Button
var button_disable_reverb_music: Button
var button_enable_dist_music: Button
var button_disable_dist_music: Button
var button_increase_pitch: Button
var button_decrease_pitch: Button
var effects_label_music: RichTextLabel


func _ready() -> void:
    var vbox: VBoxContainer = VBoxContainer.new()
    add_child(vbox)
    var viewport: Window = get_viewport() as Window
    if viewport is Window:
        vbox.offset_left = -viewport.size.x / 2.0
        vbox.offset_top = -viewport.size.y / 2.0

    option_button_music = OptionButton.new()
    for path: String in music_list:
        option_button_music.add_item(path.get_file())
    vbox.add_child(option_button_music)

    button_play_music = _create_button("Play Music", _on_button_play_music_pressed)
    vbox.add_child(button_play_music)

    button_stop_all_music = _create_button("Stop All Music", _on_button_stop_all_music_pressed)
    vbox.add_child(button_stop_all_music)

    button_enable_reverb_music = _create_button(
        "Enable Reverb", _on_button_enable_reverb_music_pressed
    )
    vbox.add_child(button_enable_reverb_music)

    button_disable_reverb_music = _create_button(
        "Disable Reverb", _on_button_disable_reverb_music_pressed
    )
    vbox.add_child(button_disable_reverb_music)

    button_enable_dist_music = _create_button(
        "Enable Distortion", _on_button_enable_dist_music_pressed
    )
    vbox.add_child(button_enable_dist_music)

    button_disable_dist_music = _create_button(
        "Disable Distortion", _on_button_disable_dist_music_pressed
    )
    vbox.add_child(button_disable_dist_music)

    button_increase_pitch = _create_button("Increase Pitch", _on_button_increase_pitch_pressed)
    vbox.add_child(button_increase_pitch)

    button_decrease_pitch = _create_button("Decrease Pitch", _on_button_decrease_pitch_pressed)
    vbox.add_child(button_decrease_pitch)

    effects_label_music = RichTextLabel.new()
    effects_label_music.text = "Active Effects: None"
    vbox.add_child(effects_label_music)


func _create_button(text: String, callback: Callable) -> Button:
    var button: Button = Button.new()
    button.text = text
    button.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
    button.pressed.connect(callback)
    return button


func _on_button_play_music_pressed() -> void:
    var selected_index: int = option_button_music.get_selected_id()
    var music_path: String = music_list[selected_index]
    var music_res: AudioStream = load(music_path) as AudioStream
    if music_res:
        AudioManager.play_music(music_res, 1.0)
        print("Playing: " + music_path)


func _on_button_stop_all_music_pressed() -> void:
    AudioManager.stop_music()
    print("All music stopped.")


func _on_button_enable_reverb_music_pressed() -> void:
    AudioEffects.add_reverb(AudioBus.BUS.MUSIC)
    effects_label_music.text = "Active Effects: Reverb Enabled"


func _on_button_disable_reverb_music_pressed() -> void:
    AudioEffects.remove_effect(AudioBus.BUS.MUSIC, "AudioEffectReverb")
    effects_label_music.text = "Active Effects: None"


func _on_button_enable_dist_music_pressed() -> void:
    AudioEffects.add_distortion(AudioBus.BUS.MUSIC)
    effects_label_music.text = "Active Effects: Distortion Enabled"


func _on_button_disable_dist_music_pressed() -> void:
    AudioEffects.remove_effect(AudioBus.BUS.MUSIC, "AudioEffectDistortion")
    effects_label_music.text = "Active Effects: None"


func _on_button_increase_pitch_pressed() -> void:
    current_pitch += PITCH_STEP
    current_pitch = clamp(current_pitch, MIN_PITCH, MAX_PITCH)
    AudioEffects.set_pitch_shift(AudioBus.BUS.MUSIC, current_pitch)
    print("Pitch increased to: ", current_pitch)


func _on_button_decrease_pitch_pressed() -> void:
    current_pitch -= PITCH_STEP
    current_pitch = clamp(current_pitch, MIN_PITCH, MAX_PITCH)
    AudioEffects.set_pitch_shift(AudioBus.BUS.MUSIC, current_pitch)
    print("Pitch decreased to: ", current_pitch)
