extends Resource
class_name SoundFont

@export var sound_font_file_path: String = ""
@export var preset_bank_number: int = 0
@export var preset_patch_number: int = 0

@export var preset_name: String = ""
@export var key_range_low_midi: int = 0
@export var key_range_high_midi: int = 127
@export var lowest_note_hz: float = 0.0
@export var highest_note_hz: float = 0.0


func load_metadata() -> void:
    preset_name = MusicDimensionsManager.rust_util.get_preset_name(
        sound_font_file_path, preset_bank_number, preset_patch_number
    )

    var key_range: PackedInt32Array = MusicDimensionsManager.rust_util.get_preset_key_range(
        sound_font_file_path, preset_bank_number, preset_patch_number
    )
    key_range_low_midi = key_range[0]
    key_range_high_midi = key_range[1]
    lowest_note_hz = _midi_note_to_frequency(key_range_low_midi)
    highest_note_hz = _midi_note_to_frequency(key_range_high_midi)


static func _midi_note_to_frequency(midi_note: int) -> float:
    # A4 = MIDI 69 = 440 Hz
    return 440.0 * pow(2.0, float(midi_note - 69) / 12.0)
