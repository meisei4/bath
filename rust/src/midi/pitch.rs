use crate::midi::util::{
    midi_note_to_hsv, parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes, render_midi_to_wav_bytes,
    sample_active_notes_at_time, update_note_log_history, MidiNote,
};
use std::path::Path;
use std::{collections::HashMap, fs, string::String, vec::Vec};

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
    pub fn resolve_payload_to_midi_buffer(&mut self, midi_bytes: &[u8]) {
        self.note_buffer = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(midi_bytes);
    }

    pub fn resolve_payload_to_pcm_buffer(
        &self,
        sample_rate: i32,
        channels: u16,
        midi_bytes: &[u8],
        sf2_bytes: &[u8],
    ) -> Vec<u8> {
        render_midi_to_wav_bytes(sample_rate, channels, midi_bytes, sf2_bytes, TARGET_CHANNEL, PROGRAM)
            .expect("Failed to render MIDI to WAV")
    }

    pub fn resolve_payload_to_pcm_buffer_cache(
        &self,
        sample_rate: i32,
        channels: u16,
        midi_bytes: &[u8],
        sf2_bytes: &[u8],
        cache_path: &str,
    ) -> Vec<u8> {
        match fs::read(cache_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                let max_time = self
                    .note_buffer
                    .values()
                    .flat_map(|events| events.iter().map(|&(_on, off)| off))
                    .fold(0.0_f32, f32::max);

                let est_bytes = (max_time * sample_rate as f32 * channels as f32 * 2.0) as f64;
                let est_mb = est_bytes / (1024.0 * 1024.0);

                println!(
                    "Generating WAV cache... \n\
                 • midi size:            {} bytes\n\
                 • soundfont size:       {} bytes\n\
                 • sample rate:          {} Hz\n\
                 • channels:             {}\n\
                 • estimated duration:   {:.2} sec\n\
                 • estimated WAV size:   {:.2} MB\n\
                 → writing to:           {}\n",
                    midi_bytes.len(),
                    sf2_bytes.len(),
                    sample_rate,
                    channels,
                    max_time,
                    est_mb,
                    cache_path,
                );
                let bytes = self.resolve_payload_to_pcm_buffer(sample_rate, channels, midi_bytes, sf2_bytes);
                let actual_mb = bytes.len() as f64 / 1024.0 / 1024.0;
                println!("→ actual WAV size on disk: {:.2} MB", actual_mb);
                if let Some(parent_dir) = Path::new(cache_path).parent() {
                    let _ = fs::create_dir_all(parent_dir);
                }
                fs::write(cache_path, &bytes).expect("Failed to write WAV cache");
                bytes
            },
        }
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
}
