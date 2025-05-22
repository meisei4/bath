extends Node
class_name PitchDimension

var midi_note_on_off_event_buffer: Dictionary
var _song_time: float = 0.0
var _last_time: float = 0.0


func _ready() -> void:
    _song_time = 0.0
    _last_time = 0.0
    midi_note_on_off_event_buffer = (
        MusicDimensionsManager.rust_util.get_midi_note_on_off_event_buffer_seconds()
        as Dictionary[Vector2i, PackedVector2Array]
    )
    var wav_bytes: PackedByteArray = (
        MusicDimensionsManager
        . rust_util
        . render_midi_to_wav_bytes_constant_time(int(MusicDimensionsManager.SAMPLE_RATE))
    )
    var wav: AudioStreamWAV = AudioStreamWAV.new()
    wav.format = AudioStreamWAV.FORMAT_16_BITS
    wav.mix_rate = MusicDimensionsManager.SAMPLE_RATE
    wav.stereo = true
    wav.data = wav_bytes
    AudioPoolManager.play_music(wav)


func _process(delta_time: float) -> void:
    _song_time += delta_time
    for key: Vector2i in midi_note_on_off_event_buffer.keys():
        var note: int = int(key.x)
        var note_on_off_data: PackedVector2Array = midi_note_on_off_event_buffer[key]
        for note_on_off: Vector2 in note_on_off_data:
            var onset_timestamp: float = note_on_off.x
            if onset_timestamp > _last_time and onset_timestamp <= _song_time:
                var frequency: float = sample_pitch_at_time(onset_timestamp)
                print("▶ Note %d @ %.3f s → %.2f Hz" % [note, onset_timestamp, frequency])
    _last_time = _song_time


func sample_pitch_at_time(query_time: float) -> float:
    var last_note: int = -1
    for key: Vector2i in midi_note_on_off_event_buffer.keys():
        var note: int = int(key.x)
        var note_on_off_data: PackedVector2Array = midi_note_on_off_event_buffer[key]
        for note_on_off: Vector2 in note_on_off_data:
            var onset_timestamp: float = note_on_off.x
            if onset_timestamp > query_time:
                break
            last_note = note
    if last_note < 0:
        return 0.0
    return 440.0 * pow(2.0, float(last_note - 69) / 12.0)
