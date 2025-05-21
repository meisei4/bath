use crate::midi::keys::{key_bindings, render};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use rdev::{Event, EventType, Key};
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use std::{
    collections::HashSet,
    process::{Child, Command},
    thread,
    time::Duration,
};

pub fn launch_fluidsynth_with_font(sf2_path: &str) -> Child {
    Command::new("fluidsynth")
        .arg("-a")
        .arg("coreaudio")
        .arg("-m")
        .arg("coremidi")
        .arg(sf2_path)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("Failed to start fluidsynth: {}", e);
            exit(1);
        })
}

pub fn connect_to_first_midi_port() -> (MidiOutput, MidiOutputPort) {
    let midi_out = MidiOutput::new("rust-midi").unwrap();
    let mut attempts = 10;
    while attempts > 0 {
        let ports = midi_out.ports();
        if let Some(port) = ports.get(0) {
            return (midi_out, port.clone());
        }
        thread::sleep(Duration::from_millis(300));
        attempts -= 1;
    }
    eprintln!("No MIDI ports found.");
    exit(1);
}

pub fn handle_key_event(
    event: Event,
    connection: &mut MidiOutputConnection,
    active_keys: &mut HashSet<Key>,
) {
    match event.event_type {
        EventType::KeyPress(Key::Escape) => exit(0),
        EventType::KeyPress(key) => {
            if let Some(note) = map_key_to_midi_note(key) {
                if active_keys.insert(key) {
                    let _ = connection.send(&[0x90, note, 100]);
                }
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(note) = map_key_to_midi_note(key) {
                if active_keys.remove(&key) {
                    let _ = connection.send(&[0x80, note, 0]);
                }
            }
        }
        _ => {}
    }
    render(active_keys);
}

fn map_key_to_midi_note(key: Key) -> Option<u8> {
    key_bindings()
        .into_iter()
        .find(|b| b.key == key)
        .map(|b| b.midi_note)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiNote {
    pub midi_note: u8,
    pub instrument_id: u8,
}

pub fn parse_midi_events_into_note_on_off_event_buffer_TICKS(
    midi_path: &str,
) -> HashMap<MidiNote, Vec<(u64, u64)>> {
    let midi_file_bytes = fs::read(midi_path).unwrap();
    let standard_midi_file = Smf::parse(&midi_file_bytes).unwrap();

    let mut active_note_on_events: HashMap<(u8, u8), u64> = HashMap::new();
    let mut final_note_on_off_event_buffer: HashMap<MidiNote, Vec<(u64, u64)>> = HashMap::new();
    let mut current_instrument_for_channel: [u8; 16] = [0; 16];
    let mut flattened_event_list = Vec::new();

    for track in &standard_midi_file.tracks {
        let mut cumulative_tick = 0u64;
        for track_event in track {
            cumulative_tick += track_event.delta.as_int() as u64;
            flattened_event_list.push((cumulative_tick, track_event.kind.clone()));
        }
    }

    flattened_event_list.sort_unstable_by_key(|&(tick, _)| tick);

    for (event_tick_position, track_event_kind) in flattened_event_list {
        if let TrackEventKind::Midi {
            channel: midi_note_channel,
            message: midi_message,
        } = track_event_kind
        {
            let channel_index = midi_note_channel.as_int();
            match midi_message {
                MidiMessage::ProgramChange { program } => {
                    current_instrument_for_channel[channel_index as usize] = program.as_int();
                }
                MidiMessage::NoteOn { key, vel } => {
                    handle_note_message_TICKS(
                        channel_index,
                        key.as_int(),
                        vel.as_int(),
                        event_tick_position,
                        &current_instrument_for_channel,
                        &mut active_note_on_events,
                        &mut final_note_on_off_event_buffer,
                    );
                }
                MidiMessage::NoteOff { key, .. } => {
                    handle_note_message_TICKS(
                        channel_index,
                        key.as_int(),
                        0,
                        event_tick_position,
                        &current_instrument_for_channel,
                        &mut active_note_on_events,
                        &mut final_note_on_off_event_buffer,
                    );
                }
                _ => {}
            }
        }
    }

    let ticks_per_quarter: u64 = match standard_midi_file.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };
    debug_midi_note_onset_buffer(&final_note_on_off_event_buffer, ticks_per_quarter);

    final_note_on_off_event_buffer
}

fn handle_note_message_TICKS(
    midi_note_channel: u8,
    midi_note_number: u8,
    event_velocity: u8,
    event_tick_position: u64,
    current_instrument_for_channel: &[u8; 16],
    active_note_on_events: &mut HashMap<(u8, u8), u64>,
    midi_note_onset_buffer: &mut HashMap<MidiNote, Vec<(u64, u64)>>,
) {
    if event_velocity > 0 {
        active_note_on_events.insert((midi_note_channel, midi_note_number), event_tick_position);
    } else if let Some(onset_tick_position) =
        active_note_on_events.remove(&(midi_note_channel, midi_note_number))
    {
        let instrument_identifier = current_instrument_for_channel[midi_note_channel as usize];
        let key = MidiNote {
            midi_note: midi_note_number,
            instrument_id: instrument_identifier,
        };
        midi_note_onset_buffer
            .entry(key)
            .or_default()
            .push((onset_tick_position, event_tick_position));
    }
}

/// A duplicated version that tracks MetaMessage::Tempo and returns seconds.
pub fn parse_midi_events_into_note_on_off_event_buffer_SECONDS(
    midi_path: &str,
) -> HashMap<MidiNote, Vec<(f64, f64)>> {
    let midi_file_bytes = fs::read(midi_path).unwrap();
    let standard_midi_file = Smf::parse(&midi_file_bytes).unwrap();

    // pulses-per-quarter for tick→sec conversion
    let ticks_per_quarter: u64 = match standard_midi_file.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };

    let mut current_us_per_quarter = 500_000u64; // default = 120 BPM
    let mut last_tick = 0u64;
    let mut elapsed_seconds = 0f64;

    let mut active_note_on_events: HashMap<(u8, u8), f64> = HashMap::new();
    let mut final_note_on_off_event_buffer: HashMap<MidiNote, Vec<(f64, f64)>> = HashMap::new();
    let mut current_instrument_for_channel: [u8; 16] = [0; 16];
    let mut flattened_event_list = Vec::new();

    for track in &standard_midi_file.tracks {
        let mut cumulative_tick = 0u64;
        for track_event in track {
            cumulative_tick += track_event.delta.as_int() as u64;
            flattened_event_list.push((cumulative_tick, track_event.kind.clone()));
        }
    }

    flattened_event_list.sort_unstable_by_key(|&(t, _)| t);

    for (event_tick_position, track_event_kind) in flattened_event_list {
        // advance elapsed time
        let delta_ticks = event_tick_position - last_tick;
        let delta_secs = (delta_ticks as f64 / ticks_per_quarter as f64)
            * (current_us_per_quarter as f64 / 1_000_000.0);
        elapsed_seconds += delta_secs;
        last_tick = event_tick_position;

        match track_event_kind {
            TrackEventKind::Meta(MetaMessage::Tempo(us_per_quarter)) => {
                current_us_per_quarter = us_per_quarter.as_int() as u64;
            }
            TrackEventKind::Midi {
                channel: midi_note_channel,
                message: midi_message,
            } => {
                let channel_index = midi_note_channel.as_int();
                match midi_message {
                    MidiMessage::ProgramChange { program } => {
                        current_instrument_for_channel[channel_index as usize] = program.as_int();
                    }
                    MidiMessage::NoteOn { key, vel } => {
                        handle_note_message_SECONDS(
                            channel_index,
                            key.as_int(),
                            vel.as_int(),
                            elapsed_seconds,
                            &current_instrument_for_channel,
                            &mut active_note_on_events,
                            &mut final_note_on_off_event_buffer,
                        );
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        handle_note_message_SECONDS(
                            channel_index,
                            key.as_int(),
                            0,
                            elapsed_seconds,
                            &current_instrument_for_channel,
                            &mut active_note_on_events,
                            &mut final_note_on_off_event_buffer,
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    final_note_on_off_event_buffer
}

fn handle_note_message_SECONDS(
    midi_note_channel: u8,
    midi_note_number: u8,
    event_velocity: u8,
    event_time: f64,
    current_instrument_for_channel: &[u8; 16],
    active_note_on_events: &mut HashMap<(u8, u8), f64>,
    midi_note_onset_buffer: &mut HashMap<MidiNote, Vec<(f64, f64)>>,
) {
    if event_velocity > 0 {
        active_note_on_events.insert((midi_note_channel, midi_note_number), event_time);
    } else if let Some(onset_time) =
        active_note_on_events.remove(&(midi_note_channel, midi_note_number))
    {
        let instrument_identifier = current_instrument_for_channel[midi_note_channel as usize];
        let key = MidiNote {
            midi_note: midi_note_number,
            instrument_id: instrument_identifier,
        };
        midi_note_onset_buffer
            .entry(key)
            .or_default()
            .push((onset_time, event_time));
    }
}

pub fn debug_midi_note_onset_buffer(
    buffer: &HashMap<MidiNote, Vec<(u64, u64)>>,
    ticks_per_quarter: u64,
) {
    if buffer.is_empty() {
        println!("-- no note events to display --");
        return;
    }
    let bars_to_display = 8;
    let ticks_per_bar = ticks_per_quarter * 4;
    let max_tick_to_display = ticks_per_bar * bars_to_display as u64;
    let chart_width: usize = 128;
    let scale = max_tick_to_display as f64 / chart_width as f64;
    let label_width = 7;
    let segment = chart_width / bars_to_display;
    print!("{:label_width$}", "");
    for bar in 1..=bars_to_display {
        let bar_str = bar.to_string();
        print!("│{}", bar_str);
        for _ in 0..segment - 1 - bar_str.len() {
            print!("─");
        }
    }
    println!();
    fn note_name(n: u8) -> String {
        let names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let octave = n / 12;
        format!("{}{}", names[(n % 12) as usize], octave)
    }
    let mut all_notes: Vec<MidiNote> = buffer.keys().cloned().collect();
    all_notes.sort_by_key(|n| n.midi_note);
    all_notes.reverse();
    let mut pairs = Vec::new();
    let mut i = 0;
    while i < all_notes.len() {
        let top = all_notes[i].clone();
        let bottom = all_notes.get(i + 1).cloned();
        pairs.push((top, bottom));
        i += 2;
    }
    for (top, bottom_opt) in pairs {
        let label = if let Some(bottom) = &bottom_opt {
            format!(
                "{}/{}",
                note_name(top.midi_note),
                note_name(bottom.midi_note)
            )
        } else {
            note_name(top.midi_note)
        };
        print!("{:<label_width$}", label);
        let mut row = vec![' '; chart_width];
        if let Some(segments) = buffer.get(&top) {
            for &(onset, release) in segments {
                if onset >= max_tick_to_display {
                    continue;
                }
                let start = (onset as f64 / scale).floor() as usize;
                let end = ((release.min(max_tick_to_display) as f64) / scale).ceil() as usize;
                for x in start.min(chart_width - 1)..end.min(chart_width) {
                    row[x] = '▀';
                }
            }
        }
        if let Some(bottom) = bottom_opt {
            if let Some(segments) = buffer.get(&bottom) {
                for &(onset, release) in segments {
                    if onset >= max_tick_to_display {
                        continue;
                    }
                    let start = (onset as f64 / scale).floor() as usize;
                    let end = ((release.min(max_tick_to_display) as f64) / scale).ceil() as usize;
                    for x in start.min(chart_width - 1)..end.min(chart_width) {
                        row[x] = match row[x] {
                            '▀' | '█' => '█',
                            _ => '▄',
                        };
                    }
                }
            }
        }
        let line: String = row.into_iter().collect();
        println!("{}", line);
    }
}
