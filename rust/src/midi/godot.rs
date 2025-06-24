use crate::midi::util::{
    inject_program_change, parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes,
    parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes, prepare_events, process_midi_events_with_timing,
    render_sample_frame, write_samples_to_wav_bytes,
};

use godot::builtin::{Dictionary, PackedByteArray, PackedVector2Array, Vector2, Vector2i};
use godot::classes::{file_access::ModeFlags, FileAccess};
use godot::prelude::GString;
use midly::{MidiMessage, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::sync::Arc;

pub fn write_samples_to_wav(sample_rate: i32, samples: Vec<(i16, i16)>) -> PackedByteArray {
    let wav_bytes = write_samples_to_wav_bytes(sample_rate, samples);
    PackedByteArray::from(wav_bytes)
}

pub fn make_note_on_off_event_dict_ticks(midi_path: &GString) -> Dictionary {
    let gd_file = FileAccess::open(midi_path, ModeFlags::READ).unwrap();
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

pub fn make_note_on_off_event_dict_seconds(midi_path: &GString) -> Dictionary {
    let gd_file = FileAccess::open(midi_path, ModeFlags::READ).unwrap();
    let midi_bytes = gd_file.get_buffer(gd_file.get_length() as i64).to_vec();
    let note_map_secs = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(&midi_bytes);
    let mut dict = Dictionary::new();
    for (key, segments) in note_map_secs {
        let dict_key = Vector2i::new(key.midi_note as i32, key.instrument_id as i32);
        let mut arr = PackedVector2Array::new();
        for (onset, release) in segments {
            arr.push(Vector2::new(onset as f32, release as f32));
        }
        let _ = dict.insert(dict_key, arr);
    }
    dict
}

pub fn render_midi_to_sound_bytes_constant_time(
    sample_rate: i32,
    midi_file_path: &GString,
    sf2_file_path: &GString,
) -> PackedByteArray {
    let sf2_path = sf2_file_path.to_string();
    let sf2_file = FileAccess::open(&sf2_path, ModeFlags::READ).unwrap();
    let sf2_bytes = sf2_file.get_buffer(sf2_file.get_length() as i64).to_vec();
    let mut sf2_cursor = std::io::Cursor::new(sf2_bytes);
    let soundfont = Arc::new(SoundFont::new(&mut sf2_cursor).unwrap());
    let mut synth = Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate)).unwrap();
    let midi_path = midi_file_path.to_string();
    let midi_file = FileAccess::open(&midi_path, ModeFlags::READ).unwrap();
    let midi_bytes = midi_file.get_buffer(midi_file.get_length() as i64).to_vec();
    let smf = midly::Smf::parse(&midi_bytes).unwrap();
    let mut events = prepare_events(&smf);
    events = inject_program_change(events, 0, 0); // channel=0, program=0 (Accordion)
    let mut audio: Vec<(i16, i16)> = Vec::new();
    let mut active_notes = std::collections::HashSet::new();
    let mut time_cursor = 0.0;
    let step_secs = 1.0 / (sample_rate as f64);
    process_midi_events_with_timing(events, &smf, |event_time, event, ch| {
        while time_cursor < event_time {
            audio.push(render_sample_frame(&mut synth));
            time_cursor += step_secs;
        }
        if let Some(channel) = ch {
            if let TrackEventKind::Midi {
                message, ..
            } = event
            {
                match message {
                    MidiMessage::NoteOn {
                        key,
                        vel,
                    } => {
                        let note = key.as_int() as i32;
                        let velocity = vel.as_int() as i32;
                        if velocity > 0 {
                            synth.note_on(channel as i32, note, velocity);
                            active_notes.insert((channel, note));
                        } else {
                            synth.note_off(channel as i32, note);
                            active_notes.remove(&(channel, note));
                        }
                    },
                    MidiMessage::NoteOff {
                        key, ..
                    } => {
                        let note = key.as_int() as i32;
                        synth.note_off(channel as i32, note);
                        active_notes.remove(&(channel, note));
                    },
                    _ => {},
                }
            }
        }
    });
    while !active_notes.is_empty() {
        audio.push(render_sample_frame(&mut synth));
        time_cursor += step_secs;
    }
    let wav_bytes = write_samples_to_wav_bytes(sample_rate, audio);
    PackedByteArray::from(wav_bytes)
}
