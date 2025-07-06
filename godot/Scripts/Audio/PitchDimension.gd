extends Node
class_name PitchDimension

var song_time: float = 0.0
var midi: Midi
var wav_stream: AudioStreamWAV


func _ready():
    midi = Midi.new()
    midi.load_midi_to_buffer(ResourcePaths.FINGERBIB)
    setup_wav()
    AudioPoolManager.play_music(wav_stream)


func _process(delta):
    song_time += delta
    midi.update_hsv_buffer(song_time)


func get_hsv_buffer() -> PackedVector3Array:
    return midi.get_hsv_buffer()


func setup_wav():
    if ResourceLoader.exists(ResourcePaths.CACHED_WAV):
        wav_stream = load(ResourcePaths.CACHED_WAV)
    else:
        var bytes: PackedByteArray = midi.render_midi_to_sound_bytes_constant_time(
            int(AudioServer.get_mix_rate()), ResourcePaths.FINGERBIB, ResourcePaths.DSDNMOY_SF2
        )
        var file: FileAccess = FileAccess.open(ResourcePaths.CACHED_WAV, FileAccess.WRITE)
        file.store_buffer(bytes)
        file.close()
        wav_stream = AudioStreamWAV.load_from_buffer(bytes)
