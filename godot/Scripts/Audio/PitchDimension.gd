extends Node
class_name PitchDimension

#TODO: rust godot gdextension cant return typed Dictionaries yet i dont think, have to use
# dict = rust_util.function() as Dictionary[key_type, value_type] for that i guess
var midi_note_on_off_event_buffer: Dictionary  #[Vector2i, PackedVector2Array]
var _song_time: float = 0.0
var _last_time: float = 0.0

var hsv_buffer: PackedVector3Array = PackedVector3Array()

const TAU: float = 2.0 * PI
const MAX_NOTE_HISTORY: int = 6
var _last_active_notes: Array[int] = []
var _note_log_history: Array[String] = []

var wav_stream: AudioStreamWAV


func _ready() -> void:
    _song_time = 0.0
    _last_time = 0.0
    midi_note_on_off_event_buffer = (
        RustUtilSingleton.rust_util.get_midi_note_on_off_event_buffer_seconds(
            ResourcePaths.FINGERBIB
        )
        as Dictionary[Vector2i, PackedVector2Array]
    )
    #TODO: look into ogg vorbis compression for better audio file resources later, maybe OPUS too
    setup_wav()
    AudioPoolManager.play_music(wav_stream)


func setup_wav() -> void:
    #TODO: this is a wack, issue because i cant preload if the file doesnt exist,
    # but because of how slow the web build IndexedDB is for loading MB large files like wavs
    # I cant re`ally test the caching process or the midi->wav bytes process until i figure out
    # how godot's REsource Loader works with preload and load and get_buffer and all the file io
    # This is very important for resource and file io in godot that really starts to become apparent in
    # runtime builds:
    # https://docs.godotengine.org/en/stable/tutorials/best_practices/logic_preferences.html

    #TODO: this is seriously needing of a completely new approach with all this
    # caching stuff and offline builds vs web build (which parts of the rust util can wasm use etc)
    if ResourceLoader.exists(ResourcePaths.CACHED_WAV):
        wav_stream = load(ResourcePaths.CACHED_WAV) as AudioStreamWAV
    else:
        var sound_bytes: PackedByteArray = (
            RustUtilSingleton
            . rust_util
            . render_midi_to_sound_bytes_constant_time(
                int(MusicDimensionsManager.SAMPLE_RATE),
                ResourcePaths.FINGERBIB,
                ResourcePaths.DSDNMOY_SF2
            )
        )
        var file_access: FileAccess = FileAccess.open(ResourcePaths.CACHED_WAV, FileAccess.WRITE)
        file_access.store_buffer(sound_bytes)
        file_access.close()
        wav_stream = AudioStreamWAV.load_from_buffer(sound_bytes)


func _process(delta: float) -> void:
    _song_time += delta
    var active_notes: Array[int] = sample_active_notes_at_time(_song_time)
    hsv_buffer.clear()
    var max_notes: int = 6
    var buffered_notes: Array[int] = active_notes.slice(0, min(active_notes.size(), max_notes))
    for note: int in buffered_notes:
        var color: Dictionary = midi_note_to_color_dict(note, active_notes.size())
        var hsv: Vector3 = Vector3(color["pitch_radians"], color["saturation"], color["value"])
        hsv_buffer.append(hsv)

    while hsv_buffer.size() < max_notes:
        hsv_buffer.append(Vector3(0, 0, 0))

    if active_notes != _last_active_notes:
        _last_active_notes = active_notes.duplicate()
        var line: String = "time: %.3f | polyphony: %d\n" % [_song_time, active_notes.size()]
        for note: int in active_notes:
            var _name: String = midi_note_to_name(note)
            var freq: float = midi_note_to_frequency(note)
            var color: Dictionary = midi_note_to_color_dict(note, active_notes.size())
            line += (
                "  - %-4s (MIDI:%2d, %6.2fHz)  hue: %5.2frad | sat: %.2f | val: %.2f\n"
                % [_name, note, freq, color["pitch_radians"], color["saturation"], color["value"]]
            )
        _note_log_history.append(line)
        if _note_log_history.size() > MAX_NOTE_HISTORY:
            _note_log_history.pop_front()
        _debug_polyphony_buffer()
    _last_time = _song_time


func midi_note_to_color_dict(note: int, polyphony: int) -> Dictionary:
    var pitch_class: int = note % 12
    var pitch_radians: float = pitch_class / 12.0 * TAU
    var octave: int = note / 12 - 1
    var value: float = clampf((octave - 1) / 7.0, 0.3, 1.0)
    var saturation: float = clampf((polyphony) / 8.0, 0.3, 1.0)
    var freq: float = 440.0 * pow(2.0, (note - 69) / 12.0)
    var _name: String = midi_note_to_name(note)
    return {
        "note": note,
        "freq": freq,
        "name": _name,
        "pitch_radians": pitch_radians,
        "saturation": saturation,
        "value": value
    }


func sample_active_notes_at_time(query_time: float) -> Array[int]:
    var notes: Array[int] = []
    for key: Vector2i in midi_note_on_off_event_buffer.keys():
        var note: int = key.x
        var note_on_off_data: PackedVector2Array = midi_note_on_off_event_buffer[key]
        for note_on_off: Vector2 in note_on_off_data:
            var onset: float = note_on_off.x
            var release: float = note_on_off.y
            if onset <= query_time and query_time < release:
                notes.append(note)
                break
    notes.sort()
    return notes


func midi_note_to_frequency(note: int) -> float:
    return 440.0 * pow(2.0, float(note - 69) / 12.0)


func midi_note_to_name(note: int) -> String:
    var note_names: Array[String] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
    ]
    var _name: String = note_names[note % 12]
    var octave: int = (note / 12) - 1
    return "%s%d" % [_name, octave]


func print_color_note_dict(data: Dictionary) -> void:
    print(
        (
            " %s (MIDI %d, %.2f Hz) → hue: %.3f rad, sat: %.2f, val: %.2f"
            % [
                data["name"],
                data["note"],
                data["freq"],
                data["pitch_radians"],
                data["saturation"],
                data["value"]
            ]
        )
    )


func _debug_polyphony_buffer() -> void:
    clear_console()
    print("=== polyphony buffer (last %d changes) ===" % MAX_NOTE_HISTORY)
    for entry: String in _note_log_history:
        print(entry)


func clear_console() -> void:
    var escape: String = PackedByteArray([0x1b]).get_string_from_ascii()
    print(escape + "[2J")

#EXPLNANTION
#      Keys with Sharps (♯)             Keys with Flats (♭)
#         B  →  5♯ (F♯ C♯ G♯ D♯ A♯)    ♭6 ←  G♭  (B♭ E♭ A♭ D♭ G♭ C♭)
#         ↑                             ↑
#    E → 4♯                    ♭5 ← D♭
#         ↑                             ↑
#    A → 3♯                    ♭4 ← A♭
#         ↑                             ↑
#   D → 2♯                     ♭3 ← E♭
#         ↑                             ↑
#   G → 1♯                     ♭2 ← B♭
#         ↑                             ↑
#   C → 0♯♭ (natural)          ♭1 ← F
# Notes:
# - Each move right on the circle adds a sharp (♯).
# - Each move left adds a flat (♭).
# - C major is the center (no accidentals).
# - Enharmonic equivalents exist:
#     - B  = C♭
#     - F♯ = G♭
#     - D♯ = E♭, etc.

# Issues in western theory:
# - A♯ major would need double sharps (B♯, E♯, etc) → avoided
# - B♭ is preferred over A♯ in nearly all practical music
# - Same goes for E♭ over D♯, etc.
# - Use ♯ keys (G, D, A, E, etc.) → sharps like F♯, C♯
# - Use ♭ keys (F, B♭, E♭, etc.) → flats like B♭, E♭
#
# For MIDI:
# - Map enharmonics based on key context
# - Default to sharps unless in a flat key
