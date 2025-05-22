#![allow(dead_code)]
//TODO: this is very hard to strucutre becuase it needs to be shared with my main.rs testing and the lib.rs
// but there isa ton of unused code between both of them so you get compiler warnings all over the place

use crate::midi::midi::{
    connect_to_first_midi_port, prepare_events, process_midi_events_with_timing,
};
use midly::{MidiMessage, Smf, TrackEventKind};
use std::{fs, thread, time::Duration};
pub fn play_midi(midi_path: &str, sf2_path: &str, preset: &str) {
    const TARGET_CHANNEL: u8 = 0;
    const PROGRAM: u8 = 0; // Accordion
    const MIDI_NOTE_ON: u8 = 0x90;
    const MIDI_NOTE_OFF: u8 = 0x80;
    let (midi_out, port) = connect_to_first_midi_port();
    let mut conn = midi_out.connect(&port, "rust-midi").unwrap();
    //TODO: this might not even be neccessary anymore, the program change event actually assigns the instruments apparently
    // assign_midi_instrument_from_soundfont(TARGET_CHANNEL, preset, sf2_path, |msg| {
    //     conn.send(msg).ok();
    // });
    let bytes = fs::read(midi_path).unwrap();
    let smf = Smf::parse(&bytes).unwrap();
    let mut events = prepare_events(&smf);
    //events = inject_program_change(events, TARGET_CHANNEL, PROGRAM);
    let mut last_time = 0.0;
    process_midi_events_with_timing(events, &smf, |event_time, event, ch| {
        let delay = event_time - last_time;
        if delay > 0.0 {
            thread::sleep(Duration::from_secs_f64(delay));
        }
        last_time = event_time;
        if let Some(channel) = ch {
            match event {
                TrackEventKind::Midi { message, .. } => match message {
                    MidiMessage::NoteOn { key, vel } => {
                        let msg = if vel.as_int() > 0 {
                            vec![MIDI_NOTE_ON | channel, key.as_int(), vel.as_int()]
                        } else {
                            vec![MIDI_NOTE_OFF | channel, key.as_int(), 0]
                        };
                        let _ = conn.send(&msg);
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        let msg = vec![MIDI_NOTE_OFF | channel, key.as_int(), 0];
                        let _ = conn.send(&msg);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    });
}
