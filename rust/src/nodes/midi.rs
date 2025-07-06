use crate::midi::util::{
    midi_note_to_hsv, parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes, render_midi_to_wav_bytes,
    sample_active_notes_at_time, update_note_log_history, MidiNote,
};
use godot::builtin::{GString, PackedByteArray, PackedVector3Array, Vector3};
use godot::classes::file_access::ModeFlags;
use godot::classes::{FileAccess, Node};
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};
use std::collections::HashMap;

// godot --path . --scene Scenes/Audio/PitchDimension.tscn
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct Midi {
    #[base]
    base: Base<Node>,
    note_buffer: HashMap<MidiNote, Vec<(f32, f32)>>,
    last_active_notes: Vec<u8>,
    note_log_history: Vec<String>,
    hsv_buffer: Vec<[f32; 3]>,
}

const TARGET_CHANNEL: u8 = 0;
const PROGRAM: u8 = 0; //"Accordion" figure out a better way to do this

#[godot_api]
impl Midi {
    #[func]
    pub fn load_midi_to_buffer(&mut self, midi_file_path: GString) {
        let gd_file = FileAccess::open(&midi_file_path.to_string(), ModeFlags::READ).unwrap();
        let midi_bytes = gd_file.get_buffer(gd_file.get_length() as i64).to_vec();
        self.note_buffer = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(&midi_bytes);
    }

    #[func]
    pub fn update_hsv_buffer(&mut self, time: f32) -> PackedByteArray {
        let notes = sample_active_notes_at_time(&self.note_buffer, time);
        self.hsv_buffer.clear();
        let polyphony = notes.len();
        for note in notes.iter().take(6) {
            let (h, s, v) = midi_note_to_hsv(*note, polyphony);
            self.hsv_buffer.push([h, s, v]);
        }
        while self.hsv_buffer.len() < 6 {
            self.hsv_buffer.push([0.0, 0.0, 0.0]);
        }

        update_note_log_history(time, &notes, &mut self.last_active_notes, &mut self.note_log_history);
        PackedByteArray::from(notes)
    }

    #[func]
    pub fn get_hsv_buffer(&self) -> PackedVector3Array {
        let mut out = PackedVector3Array::new();
        for [h, s, v] in &self.hsv_buffer {
            out.push(Vector3::new(*h, *s, *v));
        }
        out
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
