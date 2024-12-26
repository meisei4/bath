extends Control
class_name AudioTest

const SFX_534_OCARINA: String = "res://Resources/Audio/SFX/534_ocarina.wav"
const SFX_114_CLASSIC: String = "res://Resources/Audio/SFX/114_classic.wav"
const SFX_115_ECHOY: String = "res://Resources/Audio/SFX/115_echoy.wav"
const SFX_127_JUSTGOODSOUND: String = "res://Resources/Audio/SFX/127_justgoodsound.wav"
const SFX_137_INDUSTRIAL: String = "res://Resources/Audio/SFX/137_industrial.wav"
const SFX_246_SHWA: String = "res://Resources/Audio/SFX/246_shwa.wav"
const SFX_247_SHWA: String = "res://Resources/Audio/SFX/247_shwa.wav"
const SFX_133_ECHOY: String = "res://Resources/Audio/SFX/133_echoy.wav"
const SFX_249_ALIEN_DOLPHIN: String = "res://Resources/Audio/SFX/249_alien_dolphin.wav"
const SFX_413_ALIEN_DOLPHIN_1: String = "res://Resources/Audio/SFX/413_alien_dolphin_1.wav"
const SFX_428_ALIEN_DOLPHIN_3: String = "res://Resources/Audio/SFX/428_alien_dolphin_3.wav"
const SFX_451_COOL_DOLPHIN_TRICK: String = "res://Resources/Audio/SFX/451_cool_dolphin_trick.wav"
const SFX_326_SAD_ALIEN_DEPRESS: String = "res://Resources/Audio/SFX/326_sad_alien_depress.wav"
const SFX_431_PAINFUL_DOLPHIN: String = "res://Resources/Audio/SFX/431_painful_dolphin.wav"
const SFX_477_DOLPHIN: String = "res://Resources/Audio/SFX/477_dolphin.wav"
const SFX_256_BAD_SOUND_BZZ: String = "res://Resources/Audio/SFX/256_bad_sound_bzz.wav"
const SFX_259_FUTURISTIC: String = "res://Resources/Audio/SFX/259_futuristic.wav"
const SFX_405_FUTURUEY: String = "res://Resources/Audio/SFX/405_futuruey.wav"
const SFX_343_ECHOY: String = "res://Resources/Audio/SFX/343_echoy.wav"
const SFX_322_ECHOY: String = "res://Resources/Audio/SFX/322_echoy.wav"
const SFX_261_SCARY_BELL: String = "res://Resources/Audio/SFX/261_scary_bell.wav"
const SFX_425_ECHOY: String = "res://Resources/Audio/SFX/425_echoy.wav"
const SFX_452_AIRPLANE_LIFT_OFF_UFO: String = "res://Resources/Audio/SFX/452_airplane_lift_off_ufo.wav"
const SFX_454_ALIEN: String = "res://Resources/Audio/SFX/454_alien.wav"
const SFX_305_BARREL_WOODEN: String = "res://Resources/Audio/SFX/305_barrel_wooden.wav"
const SFX_365_CRASH_WITH_BUZZ: String = "res://Resources/Audio/SFX/365_crash_with_buzz.wav"
const SFX_353_BUBBLY_GROSS_THICK_WATER: String = "res://Resources/Audio/SFX/353_bubbly_gross_thick_water.wav"
const SFX_379_BUBBLY_GROSS: String = "res://Resources/Audio/SFX/379_bubbly_gross.wav"
const SFX_380_BUBBLY_GROSS_2: String = "res://Resources/Audio/SFX/380_bubbly_gross_2.wav"
const SFX_383_GOOEY: String = "res://Resources/Audio/SFX/383_gooey.wav"
const SFX_433_SCARY_BLACKNESS_PORTAL: String = "res://Resources/Audio/SFX/433_scary_blackness_portal.wav"
const SFX_462_BUBBLY: String = "res://Resources/Audio/SFX/462_bubbly.wav"
const SFX_466_FUTURISTIC_BUBBLY: String = "res://Resources/Audio/SFX/466_futuristic_bubbly.wav"
const SFX_469_SPLASH: String = "res://Resources/Audio/SFX/469_splash.wav"
const SFX_391_FOOTSTEPS_RUNNING_AWAY: String = "res://Resources/Audio/SFX/391_footsteps_running_away.wav"
const SFX_266_SOME_SOUND: String = "res://Resources/Audio/SFX/266_some_sound.wav"
const SFX_377_SNOWY_SHWA: String = "res://Resources/Audio/SFX/377_snowy_shwa.wav"
const SFX_541_BWUOO: String = "res://Resources/Audio/SFX/541_bwuoo.wav"
const SFX_542_BWOUAAA: String = "res://Resources/Audio/SFX/542_bwouaaa.wav"
const SFX_544_METAL_ICE_SHARD: String = "res://Resources/Audio/SFX/544_metal_ice_shard.wav"
const SFX_545_METAL_ICE_SHARD_DUPLICATE: String = "res://Resources/Audio/SFX/545_metal_ice_shard.wav"
const SFX_135_BUBBLY_DOWNFALL_DROWN: String = "res://Resources/Audio/SFX/135_bubbly_downfall_drown.wav"

var sfx_list: Array[String] = [
    SFX_534_OCARINA,
    SFX_114_CLASSIC,
    SFX_115_ECHOY,
    SFX_127_JUSTGOODSOUND,
    SFX_137_INDUSTRIAL,
    SFX_246_SHWA,
    SFX_247_SHWA,
    SFX_133_ECHOY,
    SFX_249_ALIEN_DOLPHIN,
    SFX_413_ALIEN_DOLPHIN_1,
    SFX_428_ALIEN_DOLPHIN_3,
    SFX_451_COOL_DOLPHIN_TRICK,
    SFX_326_SAD_ALIEN_DEPRESS,
    SFX_431_PAINFUL_DOLPHIN,
    SFX_477_DOLPHIN,
    SFX_256_BAD_SOUND_BZZ,
    SFX_259_FUTURISTIC,
    SFX_405_FUTURUEY,
    SFX_343_ECHOY,
    SFX_322_ECHOY,
    SFX_261_SCARY_BELL,
    SFX_425_ECHOY,
    SFX_452_AIRPLANE_LIFT_OFF_UFO,
    SFX_454_ALIEN,
    SFX_305_BARREL_WOODEN,
    SFX_365_CRASH_WITH_BUZZ,
    SFX_353_BUBBLY_GROSS_THICK_WATER,
    SFX_379_BUBBLY_GROSS,
    SFX_380_BUBBLY_GROSS_2,
    SFX_383_GOOEY,
    SFX_433_SCARY_BLACKNESS_PORTAL,
    SFX_462_BUBBLY,
    SFX_466_FUTURISTIC_BUBBLY,
    SFX_469_SPLASH,
    SFX_391_FOOTSTEPS_RUNNING_AWAY,
    SFX_266_SOME_SOUND,
    SFX_377_SNOWY_SHWA,
    SFX_541_BWUOO,
    SFX_542_BWOUAAA,
    SFX_544_METAL_ICE_SHARD,
    SFX_545_METAL_ICE_SHARD_DUPLICATE,
    SFX_135_BUBBLY_DOWNFALL_DROWN
]

var option_button_sfx: OptionButton
var button_play: Button
var button_stop_all: Button
var button_enable_reverb: Button
var button_disable_reverb: Button
var button_enable_dist: Button
var button_disable_dist: Button
var active_sounds_box: VBoxContainer


func _ready() -> void:
    var vbox: VBoxContainer = VBoxContainer.new()
    add_child(vbox)
    vbox.offset_left = 0
    vbox.offset_top = -get_viewport().size.y / 3.0

    option_button_sfx = OptionButton.new()
    for path: String in sfx_list:
        option_button_sfx.add_item(path)
    vbox.add_child(option_button_sfx)

    button_play = _create_button("Play", _on_button_play_pressed)
    vbox.add_child(button_play)

    button_stop_all = _create_button("Stop All", _on_button_stop_all_pressed)
    vbox.add_child(button_stop_all)

    button_enable_reverb = _create_button("Enable Reverb", _on_button_enable_reverb_pressed)
    vbox.add_child(button_enable_reverb)

    button_disable_reverb = _create_button("Disable Reverb", _on_button_disable_reverb_pressed)
    vbox.add_child(button_disable_reverb)

    button_enable_dist = _create_button("Enable Distortion", _on_button_enable_dist_pressed)
    vbox.add_child(button_enable_dist)

    button_disable_dist = _create_button("Disable Distortion", _on_button_disable_dist_pressed)
    vbox.add_child(button_disable_dist)

    active_sounds_box = VBoxContainer.new()
    active_sounds_box.name = "Active Sounds"
    vbox.add_child(active_sounds_box)


func _create_button(text: String, callback: Callable) -> Button:
    var button: Button = Button.new()
    button.text = text
    button.size_flags_horizontal = Control.SIZE_SHRINK_END
    button.pressed.connect(callback)
    return button


func _on_button_play_pressed() -> void:
    var sfx_path: String = option_button_sfx.get_item_text(option_button_sfx.get_selected_id())
    if sfx_path:
        var sfx_res: Resource = load(sfx_path)
        if sfx_res:
            AudioManager.play_sfx(sfx_res, 0.0, "SFX")
            _update_active_sounds(sfx_path, "SFX")


func _on_button_stop_all_pressed() -> void:
    AudioManager.stop_all_sfx()
    active_sounds_box.queue_redraw()


func _on_button_enable_reverb_pressed() -> void:
    AudioEffects.add_reverb(AudioBus.BUS.SFX)


func _on_button_disable_reverb_pressed() -> void:
    AudioEffects.remove_effect(AudioBus.BUS.SFX, "AudioEffectReverb")


func _on_button_enable_dist_pressed() -> void:
    AudioEffects.add_distortion(AudioBus.BUS.SFX)


func _on_button_disable_dist_pressed() -> void:
    AudioEffects.remove_effect(AudioBus.BUS.SFX, "AudioEffectDistortion")


func _update_active_sounds(sfx_name: String, bus_name: String) -> void:
    var sound_info: HBoxContainer = HBoxContainer.new()
    var label_name: Label = Label.new()
    label_name.text = "Sound: " + sfx_name
    sound_info.add_child(label_name)
    var label_bus: Label = Label.new()
    label_bus.text = " | Bus: " + bus_name
    sound_info.add_child(label_bus)
    var stop_button: Button = Button.new()
    stop_button.text = "Stop"
    stop_button.pressed.connect(_stop_specific_sound.bind(sfx_name))
    sound_info.add_child(stop_button)
    active_sounds_box.add_child(sound_info)


func _stop_specific_sound(sfx_name: String) -> void:
    AudioManager.stop_sfx(sfx_name)
    for child in active_sounds_box.get_children():
        if child is HBoxContainer and child.get_child(0).text.find(sfx_name) != -1:
            active_sounds_box.remove_child(child)
            child.queue_free()
            break
