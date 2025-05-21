use crate::midi::midi::connect_to_first_midi_port;
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::{fs, thread, time::Duration};

pub fn play_midi(midi_path: &str) {
    let (midi_out, port) = connect_to_first_midi_port();
    let mut midi_connection = midi_out
        .connect(&port, "rust-midi-playback")
        .expect("failed to open MIDI connection");
    // force channel 0 → bank 0/patch 0 (your Accordion preset)
    // CC0 = Bank MSB, CC32 = Bank LSB, PC = Program Change
    midi_connection.send(&[0xB0, 0x00, 0x00]).ok(); // CC0 on ch0
    midi_connection.send(&[0xB0, 0x20, 0x00]).ok(); // CC32 on ch0
    midi_connection.send(&[0xC0, 0x00]).ok(); // PC to preset 0 on ch0
    let midi_file_bytes = fs::read(midi_path).unwrap();
    let standard_midi_file = Smf::parse(&midi_file_bytes).unwrap();
    let tpq = match standard_midi_file.header.timing {
        Timing::Metrical(t) => t.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };
    let mut events = Vec::new();
    for track in &standard_midi_file.tracks {
        let mut abs = 0u64;
        for e in track {
            abs += e.delta.as_int() as u64;
            events.push((abs, e.kind.clone()));
        }
    }
    events.sort_unstable_by_key(|(t, _)| *t);
    let mut last_tick = 0u64;
    let mut us_per_qn = 500_000u64; // TODO: default 120 BPM PLACE HODLER!!!
    for (tick, kind) in events {
        let dt_ticks = tick - last_tick;
        let dt_s = (dt_ticks as f64 / tpq as f64) * (us_per_qn as f64 / 1e6);
        if dt_s > 0.0 {
            thread::sleep(Duration::from_secs_f64(dt_s));
        }
        last_tick = tick;
        match kind {
            TrackEventKind::Meta(MetaMessage::Tempo(u)) => {
                us_per_qn = u.as_int() as u64;
            }
            TrackEventKind::Midi { message, .. } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    let note = key.as_int();
                    let v = vel.as_int();
                    if v > 0 {
                        let _ = midi_connection.send(&[0x90, note, v]);
                    } else {
                        let _ = midi_connection.send(&[0x80, note, 0]);
                    }
                }
                MidiMessage::NoteOff { key, .. } => {
                    let note = key.as_int();
                    let _ = midi_connection.send(&[0x80, note, 0]);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
