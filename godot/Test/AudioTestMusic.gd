extends Control
class_name AudioTestMusic

#TODO: Looping with ogg files works fine (it has been the default for ages). As jgodfrey indicated, you may need to enable looping in the import dialog if looping has been disabled for some reason.

const MUSIC_TRACK_1: String = "res://Resources/Audio/Music/trimmed_10___What_You_Want_00-40-25_00-41-40.mp3"

var musicList: Array[String] = [
    MUSIC_TRACK_1,
]

var effectsLabel: RichTextLabel

func _ready() -> void:
    setupUI()
    startMusicTest()

func setupUI() -> void:
    var vbox: VBoxContainer = VBoxContainer.new()
    vbox.anchor_left = 0.5
    vbox.anchor_top = 0.5
    vbox.anchor_right = 0.5
    vbox.anchor_bottom = 0.5
    vbox.pivot_offset = Vector2.ZERO
    add_child(vbox)

    effectsLabel = RichTextLabel.new()
    effectsLabel.text = "Active Effects: None"
    vbox.add_child(effectsLabel)

func startMusicTest() -> void:
    # Play the first music track
    var musicResource1: Resource = loadMusic(MUSIC_TRACK_1)
    if musicResource1:
        AudioManager.playMusic(musicResource1, 0.8)
        print("Playing Music Track 1")
        _updateEffectsDisplay()

    ## Apply Reverb Effect after 2 seconds
    #await get_tree().create_timer(2.0).timeout
    #var reverbEffect: AudioEffectReverb = AudioEffectReverb.new()
    #reverbEffect.room_size = 0.9
    #reverbEffect.damping = 0.7
    #reverbEffect.wet = 0.5
    #AudioManager.addEffect("Music", reverbEffect)
    #print("Reverb Effect Added to Music Bus")
    #_updateEffectsDisplay()

    # Apply Delay Effect after 2 seconds
    #await get_tree().create_timer(2.0).timeout
    #var delayEffect: AudioEffectDelay = AudioEffectDelay.new()
    #delayEffect.tap1_active = true
    #delayEffect.tap1_delay_ms = 300.0
    #delayEffect.tap1_level_db = -6.0
    #delayEffect.tap2_active = true
    #delayEffect.tap2_delay_ms = 600.0
    #delayEffect.tap2_level_db = -12.0
    #delayEffect.feedback_active = true
    #delayEffect.feedback_delay_ms = 400.0
    #delayEffect.feedback_level_db = -6.0
    #delayEffect.feedback_lowpass_hz = 15000.0
    #delayEffect.dry = 0.8
    #AudioManager.addEffect("Music", delayEffect)
    #print("Delay Effect Added to Music Bus")
    #_updateEffectsDisplay()

    # Remove Reverb Effect after 5 seconds
    #await get_tree().create_timer(5.0).timeout
    #AudioManager.removeEffect("Music", "AudioEffectReverb")
    #print("Reverb Effect Removed from Music Bus")
    #_updateEffectsDisplay()

    ## Stop Music after 8 seconds
    #await get_tree().create_timer(8.0).timeout
    #AudioManager.stopMusic()
    #print("Music Stopped")
    #_updateEffectsDisplay()

func loadMusic(path: String) -> Resource:
    var resource: Resource = load(path)
    if resource == null:
        push_error("Failed to load music track: " + path)
    return resource

func _updateEffectsDisplay() -> void:
    var activeEffects: Array[String] = []
    var busIndex: int = AudioServer.get_bus_index("Music")
    var effectCount: int = AudioServer.get_bus_effect_count(busIndex)

    for effectIndex: int in range(effectCount):
        var currentEffect: AudioEffect = AudioServer.get_bus_effect(busIndex, effectIndex)
        activeEffects.append(currentEffect.get_class())

    if activeEffects.is_empty():
        effectsLabel.text = "Active Effects: None"
    else:
        effectsLabel.text = "Active Effects:\n" + ", ".join(activeEffects)
