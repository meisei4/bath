#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "std"))]
use heapless::LinearMap;

#[cfg(feature = "std")]
use std::{collections::HashMap, fs, string::String, vec::Vec};

use crate::midi::util::{
    midi_note_to_hsv, parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes, render_midi_to_wav_bytes,
    sample_active_notes_at_time, update_note_log_history, MidiNote,
};

#[cfg(not(feature = "std"))]
type NoteBuffer = LinearMap<MidiNote, Vec<(f32, f32)>, 128>;

#[cfg(feature = "std")]
type NoteBuffer = HashMap<MidiNote, Vec<(f32, f32)>>;

#[derive(Default)]
pub struct PitchDimension {
    note_buffer: NoteBuffer,
    last_active_notes: Vec<u8>,
    note_log_history: Vec<String>,
    hsv_buffer: Vec<[f32; 3]>,
}

const TARGET_CHANNEL: u8 = 0;
const PROGRAM: u8 = 0;

impl PitchDimension {
    pub fn load_midi_to_buffer(&mut self, midi_file_path: &str) {
        let midi_bytes =
            fs::read(midi_file_path).unwrap_or_else(|e| panic!("Failed to read MIDI file '{}': {}", midi_file_path, e));
        self.note_buffer = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(&midi_bytes);
    }

    #[cfg(not(feature = "std"))]
    pub fn load_midi_from_bytes(&mut self, midi_bytes: &[u8]) {
        self.note_buffer = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(midi_bytes);
    }

    pub fn update_hsv_buffer(&mut self, time: f32) -> Vec<u8> {
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

        notes
    }

    pub fn get_hsv_buffer(&self) -> Vec<[f32; 3]> {
        self.hsv_buffer.clone()
    }

    pub fn render_midi_to_sound_bytes_constant_time(
        &self,
        sample_rate: i32,
        midi_file_path: &str,
        sf2_file_path: &str,
    ) -> Vec<u8> {
        let sf2_bytes =
            fs::read(sf2_file_path).unwrap_or_else(|e| panic!("Failed to read SF2 file '{}': {}", sf2_file_path, e));
        let midi_bytes =
            fs::read(midi_file_path).unwrap_or_else(|e| panic!("Failed to read MIDI file '{}': {}", midi_file_path, e));
        render_midi_to_wav_bytes(sample_rate, &midi_bytes, &sf2_bytes, TARGET_CHANNEL, PROGRAM)
            .expect("Failed to render MIDI to WAV")
    }

    #[cfg(not(feature = "std"))]
    pub fn render_from_bytes(&self, sample_rate: i32, midi_bytes: &[u8], sf2_bytes: &[u8]) -> Vec<u8> {
        render_midi_to_wav_bytes(sample_rate, midi_bytes, sf2_bytes, TARGET_CHANNEL, PROGRAM).unwrap_or_default()
    }
}
