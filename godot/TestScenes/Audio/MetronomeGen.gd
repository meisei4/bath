extends Node
class_name MetronomeGen


func _ready() -> void:
    const AMPLITUDE: int = 32_767
    var sample_rate: int = MusicDimensionsManager.SAMPLE_RATE
    var DURATION: float = 0.30
    var FRAMES: int = int(sample_rate * DURATION)

    var pcm: PackedByteArray = PackedByteArray()
    pcm.resize(FRAMES * 2)
    pcm[0] = AMPLITUDE & 0xff
    pcm[1] = (AMPLITUDE >> 8) & 0xff

    var wav: AudioStreamWAV = AudioStreamWAV.new()
    wav.format = AudioStreamWAV.FORMAT_16_BITS
    wav.mix_rate = sample_rate
    wav.stereo = false
    wav.data = pcm

    var res_dir: String = "res://Resources/Audio"
    var abs_dir: String = ProjectSettings.globalize_path(res_dir)
    DirAccess.make_dir_recursive_absolute(abs_dir)

    var path: String = abs_dir + "/metronome_click.wav"
    var err: Error = wav.save_to_wav(path)
    if err != OK:
        push_error("save_to_wav failed (%d) -> %s" % [err, path])
    else:
        print("Metronome click written to ", path)
