extends Node


func add_distortion(
    bus_name: String,
    mode: AudioEffectDistortion.Mode = AudioEffectDistortion.MODE_CLIP,
    drive: float = 0.5,
    pre_gain_db: float = 0.0,
    post_gain_db: float = 0.0,
    keep_hf: float = 16000.0
) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    var distortion: AudioEffectDistortion = AudioEffectDistortion.new()
    distortion.mode = mode
    distortion.drive = drive
    distortion.pre_gain = pre_gain_db
    distortion.post_gain = post_gain_db
    distortion.keep_hf_hz = keep_hf
    AudioServer.add_bus_effect(bus_idx, distortion)


func add_delay(
    bus_name: String,
    tap1_delay_ms: float = 250.0,
    tap1_level_db: float = -6.0,
    tap1_pan: float = 0.0,
    tap2_delay_ms: float = 500.0,
    tap2_level_db: float = -12.0,
    tap2_pan: float = 0.0,
    feedback_active: bool = false,
    feedback_delay_ms: float = 340.0,
    feedback_level_db: float = -6.0,
    feedback_lowpass_hz: float = 16000.0,
    dry: float = 1.0
) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    var delay_fx: AudioEffectDelay = AudioEffectDelay.new()

    delay_fx.tap1_active = true
    delay_fx.tap1_delay_ms = tap1_delay_ms
    delay_fx.tap1_level_db = tap1_level_db
    delay_fx.tap1_pan = tap1_pan

    delay_fx.tap2_active = true
    delay_fx.tap2_delay_ms = tap2_delay_ms
    delay_fx.tap2_level_db = tap2_level_db
    delay_fx.tap2_pan = tap2_pan

    delay_fx.feedback_active = feedback_active
    delay_fx.feedback_delay_ms = feedback_delay_ms
    delay_fx.feedback_level_db = feedback_level_db
    delay_fx.feedback_lowpass = feedback_lowpass_hz

    delay_fx.dry = dry

    AudioServer.add_bus_effect(bus_idx, delay_fx)


func add_reverb(
    bus_name: String,
    room_size: float = 0.8,
    damping: float = 0.5,
    wet: float = 0.5,
    dry: float = 1.0,
    hipass: float = 0.0,
    predelay_msec: float = 150.0,
    predelay_feedback: float = 0.4,
    spread: float = 1.0
) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    var reverb: AudioEffectReverb = AudioEffectReverb.new()
    reverb.room_size = room_size
    reverb.damping = damping
    reverb.wet = wet
    reverb.dry = dry
    reverb.hipass = hipass
    reverb.predelay_msec = predelay_msec
    reverb.predelay_feedback = predelay_feedback
    reverb.spread = spread

    AudioServer.add_bus_effect(bus_idx, reverb)


func add_chorus(
    bus_name: String,
    voice_count: int = 2,
    voice_params: Dictionary = {
        0:
        {
            "delay_ms": 15.0,
            "depth_ms": 2.0,
            "rate_hz": 0.8,
            "level_db": 0.0,
            "pan": -0.5,
            "cutoff_hz": 8000.0
        },
        1:
        {
            "delay_ms": 20.0,
            "depth_ms": 3.0,
            "rate_hz": 1.2,
            "level_db": 0.0,
            "pan": 0.5,
            "cutoff_hz": 8000.0
        },
    },
    wet: float = 0.5,
    dry: float = 1.0
) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    var chorus: AudioEffectChorus = AudioEffectChorus.new()

    chorus.voice_count = clamp(voice_count, 1, 4)
    #TODO: all these type issues with the dictionaries will be resolved in next godot version i think
    for voice_idx: Key in range(chorus.voice_count):
        if voice_params.has(voice_idx):
            var vp: Dictionary = voice_params[voice_idx]
            if vp.has("delay_ms"):
                chorus.set_voice_delay_ms(voice_idx, vp["delay_ms"])
            if vp.has("depth_ms"):
                chorus.set_voice_depth_ms(voice_idx, float(vp["depth_ms"]))
            if vp.has("rate_hz"):
                chorus.set_voice_rate_hz(voice_idx, float(vp["rate_hz"]))
            if vp.has("level_db"):
                chorus.set_voice_level_db(voice_idx, float(vp["level_db"]))
            if vp.has("pan"):
                chorus.set_voice_pan(voice_idx, float(vp["pan"]))
            if vp.has("cutoff_hz"):
                chorus.set_voice_cutoff_hz(voice_idx, float(vp["cutoff_hz"]))

    chorus.wet = wet
    chorus.dry = dry

    AudioServer.add_bus_effect(bus_idx, chorus)


func remove_effect(bus_name: String, effect_type: String) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    var effect_count: int = AudioServer.get_bus_effect_count(bus_idx)
    for i: int in range(effect_count):
        var fx: AudioEffect = AudioServer.get_bus_effect(bus_idx, i)
        if fx.get_class() == effect_type:
            AudioServer.remove_bus_effect(bus_idx, i)
            return


func clear_effects(bus_name: String) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    while AudioServer.get_bus_effect_count(bus_idx) > 0:
        AudioServer.remove_bus_effect(bus_idx, 0)
