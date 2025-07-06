use crate::audio_analysis::godot::{detect_bpm_aubio_ogg, detect_bpm_aubio_wav};
use crate::midi::util::{
    parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes,
    parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes, render_midi_to_wav_bytes,
};
use godot::builtin::{Dictionary, GString, PackedByteArray, PackedVector2Array, Vector2, Vector2i};
use godot::classes::file_access::ModeFlags;
use godot::classes::{FileAccess, Node};
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct Midi {
    #[base]
    base: Base<Node>,
}

const TARGET_CHANNEL: u8 = 0;
const PROGRAM: u8 = 0; //"Accordion" figure out a better way to do this

#[godot_api]
impl Midi {
    #[func]
    pub fn detect_bpm_wav(&self, wav_file_path: GString) -> f32 {
        let wav_path = wav_file_path.to_string();
        let wav_file = FileAccess::open(&wav_path, ModeFlags::READ).unwrap();
        let wav_bytes = wav_file.get_buffer(wav_file.get_length() as i64).to_vec();
        detect_bpm_aubio_wav(&wav_bytes)
    }

    #[func]
    pub fn detect_bpm_ogg(&self, ogg_file_path: GString) -> f32 {
        let ogg_path = ogg_file_path.to_string();
        let ogg_file = FileAccess::open(&ogg_path, ModeFlags::READ).unwrap();
        let ogg_bytes = ogg_file.get_buffer(ogg_file.get_length() as i64).to_vec();
        detect_bpm_aubio_ogg(&ogg_bytes)
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_ticks(&self, midi_file_path: GString) -> Dictionary {
        let gd_file = FileAccess::open(&midi_file_path, ModeFlags::READ).unwrap();
        let midi_bytes = gd_file.get_buffer(gd_file.get_length() as i64).to_vec();
        let note_map_ticks = parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes(&midi_bytes);
        let mut dict = Dictionary::new();
        for (key, segments) in note_map_ticks {
            let dict_key = Vector2i::new(key.midi_note as i32, key.instrument_id as i32);
            let mut arr = PackedVector2Array::new();
            for (onset, release) in segments {
                arr.push(Vector2::new(onset as f32, release as f32));
            }
            let _ = dict.insert(dict_key, arr);
        }
        dict
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_seconds(&self, midi_file_path: GString) -> Dictionary {
        let gd_file = FileAccess::open(&midi_file_path, ModeFlags::READ).unwrap();
        let midi_bytes = gd_file.get_buffer(gd_file.get_length() as i64).to_vec();
        let note_map_secs = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(&midi_bytes);
        let mut dict = Dictionary::new();
        for (key, segments) in note_map_secs {
            let dict_key = Vector2i::new(key.midi_note as i32, key.instrument_id as i32);
            let mut arr = PackedVector2Array::new();
            for (onset, release) in segments {
                arr.push(Vector2::new(onset, release));
            }
            let _ = dict.insert(dict_key, arr);
        }
        dict
    }

    #[func]
    pub fn render_midi_to_sound_bytes_constant_time(
        &self,
        sample_rate: i32,
        midi_file_path: GString,
        sf2_file_path: GString,
    ) -> PackedByteArray {
        let sf2_path = sf2_file_path.to_string();
        let sf2_file = FileAccess::open(&sf2_path, ModeFlags::READ).unwrap();
        let sf2_bytes = sf2_file.get_buffer(sf2_file.get_length() as i64).to_vec();

        let midi_path = midi_file_path.to_string();
        let mid_file = FileAccess::open(&midi_path, ModeFlags::READ).unwrap();
        let midi_bytes = mid_file.get_buffer(mid_file.get_length() as i64).to_vec();

        let wav = render_midi_to_wav_bytes(sample_rate, &midi_bytes, &sf2_bytes, TARGET_CHANNEL, PROGRAM).unwrap();
        PackedByteArray::from(wav)
    }
}
