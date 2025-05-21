extends Node
class_name PitchDimension

var midi_note_on_off_event_buffer: Dictionary  #[Vector2i, PackedVector2Array] = {}
var _song_time: float = 0.0
var _last_time: float = 0.0

var _pcm: PackedFloat32Array
var _pcm_idx: int = 0
var _generator: AudioStreamGenerator
var _player: AudioStreamPlayer
var _playback: AudioStreamGeneratorPlayback


func _ready() -> void:
    # 1) Fetch your debug buffer
    midi_note_on_off_event_buffer = (
        MusicDimensionsManager.rust_util.get_midi_note_on_off_event_buffer_SECONDS()
        as Dictionary[Vector2i, PackedVector2Array]
    )

    _pcm = MusicDimensionsManager.rust_util.render_midi_to_pcm(
        int(MusicDimensionsManager.SAMPLE_RATE)
    )
    _generator = AudioStreamGenerator.new()
    _generator.mix_rate = MusicDimensionsManager.SAMPLE_RATE
    _generator.buffer_length = 0.1
    AudioPoolManager.play_music(_generator)
    for p in AudioPoolManager.music_pool.players:
        if p.stream == _generator:
            _player = p
            _playback = _player.get_stream_playback()
            break
    _fill_buffer()


func _process(delta_time: float) -> void:
    _fill_buffer()
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


func _fill_buffer() -> void:
    var avail = _playback.get_frames_available()
    for i in range(avail):
        if _pcm_idx >= _pcm.size():
            return
        var s = _pcm[_pcm_idx]
        _playback.push_frame(Vector2(s, s))
        _pcm_idx += 1


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
