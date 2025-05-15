extends Control
class_name SFXTest

var sfx_list: Array[String] = [
    AudioConsts.SFX_534_OCARINA,
    AudioConsts.SFX_114_CLASSIC_PUZZLE_SOUND,
    AudioConsts.SFX_115_ECHOY,
    AudioConsts.SFX_127_JUSTGOODSOUND,
    AudioConsts.SFX_137_INDUSTRIAL,
    AudioConsts.SFX_246_SHWA,
    AudioConsts.SFX_247_SHWA,
    AudioConsts.SFX_133_ECHOY,
    AudioConsts.SFX_249_ALIEN_DOLPHIN,
    AudioConsts.SFX_413_ALIEN_DOLPHIN_1,
    AudioConsts.SFX_428_ALIEN_DOLPHIN_3,
    AudioConsts.SFX_451_COOL_DOLPHIN_TRICK,
    AudioConsts.SFX_326_SAD_ALIEN_DEPRESS,
    AudioConsts.SFX_431_PAINFUL_DOLPHIN,
    AudioConsts.SFX_477_DOLPHIN,
    AudioConsts.SFX_256_BAD_SOUND_BZZ,
    AudioConsts.SFX_259_FUTURISTIC,
    AudioConsts.SFX_405_FUTURUEY,
    AudioConsts.SFX_343_ECHOY,
    AudioConsts.SFX_322_ECHOY,
    AudioConsts.SFX_261_SCARY_BELL,
    AudioConsts.SFX_425_ECHOY,
    AudioConsts.SFX_452_AIRPLANE_LIFT_OFF_UFO,
    AudioConsts.SFX_454_ALIEN,
    AudioConsts.SFX_305_BARREL_WOODEN,
    AudioConsts.SFX_365_CRASH_WITH_BUZZ,
    AudioConsts.SFX_353_BUBBLY_GROSS_THICK_WATER,
    AudioConsts.SFX_379_BUBBLY_GROSS,
    AudioConsts.SFX_380_BUBBLY_GROSS_2,
    AudioConsts.SFX_383_GOOEY,
    AudioConsts.SFX_433_SCARY_BLACKNESS_PORTAL,
    AudioConsts.SFX_462_BUBBLY,
    AudioConsts.SFX_466_FUTURISTIC_BUBBLY,
    AudioConsts.SFX_469_SPLASH,
    AudioConsts.SFX_391_FOOTSTEPS_RUNNING_AWAY,
    AudioConsts.SFX_266_SOME_SOUND,
    AudioConsts.SFX_377_SNOWY_SHWA,
    AudioConsts.SFX_541_BWUOO,
    AudioConsts.SFX_542_BWOUAAA,
    AudioConsts.SFX_544_METAL_ICE_SHARD,
    AudioConsts.SFX_545_METAL_ICE_SHARD_DUPLICATE,
    AudioConsts.SFX_135_BUBBLY_DOWNFALL_DROWN
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
    var viewport: Window = get_viewport() as Window
    if viewport is Window:
        vbox.offset_top = -viewport.size.y / 3.0

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
        var sfx_res: AudioStream = load(sfx_path) as AudioStream
        if sfx_res:
            AudioPoolManager.play_sfx(sfx_res)
            _update_active_sounds(sfx_path, "AudioConsts.SFX")


func _on_button_stop_all_pressed() -> void:
    AudioPoolManager.stop_all_sfx()
    active_sounds_box.queue_redraw()


func _on_button_enable_reverb_pressed() -> void:
    AudioEffectManager.add_reverb(AudioBus.BUS.SFX)


func _on_button_disable_reverb_pressed() -> void:
    AudioEffectManager.remove_effect(AudioBus.BUS.SFX, "AudioEffectReverb")


func _on_button_enable_dist_pressed() -> void:
    AudioEffectManager.add_distortion(AudioBus.BUS.SFX)


func _on_button_disable_dist_pressed() -> void:
    AudioEffectManager.remove_effect(AudioBus.BUS.SFX, "AudioEffectDistortion")


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
    AudioPoolManager.stop_sfx(sfx_name)
    for child: Node in active_sounds_box.get_children():
        if child is HBoxContainer:
            var h_box_container: HBoxContainer = child as HBoxContainer
            var button: Button = h_box_container.get_child(0) as Button
            #TODO: this is already ridiculous, shouldnt have to do this maddness with constant typing
            if button.text.find(sfx_name) != -1:
                active_sounds_box.remove_child(child)
                child.queue_free()
                break
