extends Node

@export var bus: AudioBus.BUS = AudioBus.BUS.MASTER

const DEFAULT_PITCH_SHIFT: Dictionary = {
    "pitch_scale": 1.0,
}

const DEFAULT_DISTORTION: Dictionary = {
    "mode": AudioEffectDistortion.MODE_CLIP,
    "drive": 0.5,
    "pre_gain_db": 0.0,
    "post_gain_db": 0.0,
    "keep_hf_hz": 16000.0
}

const DEFAULT_REVERB: Dictionary = {
    "room_size": 0.8,
    "damping": 0.5,
    "wet": 0.5,
    "dry": 1.0,
    "hipass": 0.0,
    "predelay_msec": 150.0,
    "predelay_feedback": 0.4,
    "spread": 1.0
}


func _get_bus_index(bus_enum: AudioBus.BUS) -> int:
    var bus_name = AudioBus.val(bus_enum)
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    if bus_idx == -1:
        push_warning("Bus not found: " + bus_name)
    return bus_idx


func _add_effect(bus_enum: AudioBus.BUS, effect: AudioEffect) -> void:
    var bus_idx: int = _get_bus_index(bus_enum)
    if bus_idx == -1:
        return
    AudioServer.add_bus_effect(bus_idx, effect)
    print("Added ", effect.get_class(), " effect to bus: ", bus_enum)


func remove_effect(bus_enum: AudioBus.BUS, effect_type: String) -> void:
    var bus_idx: int = _get_bus_index(bus_enum)
    if bus_idx == -1:
        return
    var effect_count: int = AudioServer.get_bus_effect_count(bus_idx)
    for i in range(effect_count):
        var fx: AudioEffect = AudioServer.get_bus_effect(bus_idx, i)
        if fx.get_class() == effect_type:
            AudioServer.remove_bus_effect(bus_idx, i)
            print("Removed ", effect_type, " from bus: ", bus_enum)
            return


func add_distortion(bus_enum: AudioBus.BUS, config: Dictionary = DEFAULT_DISTORTION) -> void:
    var distortion = AudioEffectDistortion.new()
    distortion.mode = config["mode"]
    distortion.drive = config["drive"]
    distortion.pre_gain = config["pre_gain_db"]
    distortion.post_gain = config["post_gain_db"]
    distortion.keep_hf_hz = config["keep_hf_hz"]
    _add_effect(bus_enum, distortion)


func add_reverb(bus_enum: AudioBus.BUS, config: Dictionary = DEFAULT_REVERB) -> void:
    var reverb = AudioEffectReverb.new()
    reverb.room_size = config["room_size"]
    reverb.damping = config["damping"]
    reverb.wet = config["wet"]
    reverb.dry = config["dry"]
    reverb.hipass = config["hipass"]
    reverb.predelay_msec = config["predelay_msec"]
    reverb.predelay_feedback = config["predelay_feedback"]
    reverb.spread = config["spread"]
    _add_effect(bus_enum, reverb)


func set_pitch_shift(bus_enum: AudioBus.BUS, pitch: float) -> void:
    var bus_idx: int = _get_bus_index(bus_enum)
    if bus_idx == -1:
        return

    var pitch_shift_found: bool = false
    for i in range(AudioServer.get_bus_effect_count(bus_idx)):
        var effect = AudioServer.get_bus_effect(bus_idx, i)
        if effect is AudioEffectPitchShift:
            effect.pitch_scale = pitch
            pitch_shift_found = true
            print("Updated pitch shift on bus ", bus_enum, " to pitch_scale: ", pitch)
            break

    if not pitch_shift_found:
        var pitch_shift = AudioEffectPitchShift.new()
        pitch_shift.pitch_scale = pitch
        _add_effect(bus_enum, pitch_shift)
        print("Added new pitch shift effect to bus ", bus_enum, " with pitch_scale: ", pitch)


func update_distortion(bus_enum: AudioBus.BUS, config: Dictionary) -> void:
    var bus_idx: int = _get_bus_index(bus_enum)
    if bus_idx == -1:
        return
    for i in range(AudioServer.get_bus_effect_count(bus_idx)):
        var fx = AudioServer.get_bus_effect(bus_idx, i)
        if fx is AudioEffectDistortion:
            if "drive" in config:
                fx.drive = config["drive"]
            if "pre_gain_db" in config:
                fx.pre_gain_db = config["pre_gain_db"]
            if "post_gain_db" in config:
                fx.post_gain_db = config["post_gain_db"]
            print("Updated distortion on bus ", bus_enum, " with config: ", config)
            break


func update_reverb(bus_enum: AudioBus.BUS, config: Dictionary) -> void:
    var bus_idx: int = _get_bus_index(bus_enum)
    if bus_idx == -1:
        return
    for i in range(AudioServer.get_bus_effect_count(bus_idx)):
        var fx = AudioServer.get_bus_effect(bus_idx, i)
        if fx is AudioEffectReverb:
            if "wet" in config:
                fx.wet = config["wet"]
            if "room_size" in config:
                fx.room_size = config["room_size"]
            if "damping" in config:
                fx.damping = config["damping"]
            print("Updated reverb on bus ", bus_enum, " with config: ", config)
            break
