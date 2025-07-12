#![cfg(feature = "tests-only")]

use asset_payload::payloads::MIDI_FILE;
use asset_payload::SOUND_FONT_FILE_PATH;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midly::{MidiMessage, Smf, TrackEventKind};
use rdev::{Event, EventType, Key};
use rustysynth::{Instrument, InstrumentRegion, Preset, SoundFont};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, BufReader, Write};
use std::process::{exit, Child, Command};
use std::thread;
use std::time::Duration;
use terminal_size::{terminal_size, Width};

use crate::midi::util::{
    parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes,
    parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes, prepare_events, process_midi_events_with_timing,
};

pub fn run_playback() -> Result<(), Box<dyn Error>> {
    print_full_structure(SOUND_FONT_FILE_PATH, 0, 0)?;
    // TODO: the below until the next TODO is commented out to play midi file
    // let mut fluidsynth_process = launch_fluidsynth_with_font(SOUND_FONT_FILE);
    // let (midi_output, midi_port) = connect_to_first_midi_port();
    // let mut midi_connection = midi_output.connect(&midi_port, "rust-midi")?;
    // let mut pressed_keys: HashSet<Key> = HashSet::new();
    // let _ = listen(move |event| {
    //     handle_key_event(event, &mut midi_connection, &mut pressed_keys);
    // });
    // let _ = fluidsynth_process.kill();
    //TODO: the above is all^^ for testing midi keyboard user input
    let midi_bytes = MIDI_FILE();
    let _ = parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes(&midi_bytes);
    let _ = parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(&midi_bytes);
    play_midi(&midi_bytes);

    Ok(())
}

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

pub fn handle_key_event(event: Event, connection: &mut MidiOutputConnection, active_keys: &mut HashSet<Key>) {
    match event.event_type {
        EventType::KeyPress(Key::Escape) => exit(0),
        EventType::KeyPress(key) => {
            if let Some(note) = map_key_to_midi_note(key) {
                if active_keys.insert(key) {
                    let _ = connection.send(&[0x90, note, 100]);
                }
            }
        },
        EventType::KeyRelease(key) => {
            if let Some(note) = map_key_to_midi_note(key) {
                if active_keys.remove(&key) {
                    let _ = connection.send(&[0x80, note, 0]);
                }
            }
        },
        _ => {},
    }
    render(active_keys);
}

fn map_key_to_midi_note(key: Key) -> Option<u8> {
    key_bindings().into_iter().find(|b| b.key == key).map(|b| b.midi_note)
}

pub struct KeyBinding {
    pub key: Key,
    pub label: char,
    pub midi_note: u8,
}

const BASE_OCTAVE: i8 = 5;
const C: u8 = 0;
const C_S: u8 = 1;
const D: u8 = 2;
const D_S: u8 = 3;
const E: u8 = 4;
const F: u8 = 5;
const F_S: u8 = 6;
const G: u8 = 7;
const G_S: u8 = 8;
const A: u8 = 9;
const A_S: u8 = 10;
const B: u8 = 11;

const fn midi_note(note_index: u8, octave: i8) -> u8 {
    (note_index as i8 + (octave * 12)) as u8
}

pub fn key_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding {
            key: Key::KeyA,
            label: 'A',
            midi_note: midi_note(C, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyW,
            label: 'W',
            midi_note: midi_note(C_S, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyS,
            label: 'S',
            midi_note: midi_note(D, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyE,
            label: 'E',
            midi_note: midi_note(D_S, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyD,
            label: 'D',
            midi_note: midi_note(E, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyF,
            label: 'F',
            midi_note: midi_note(F, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyT,
            label: 'T',
            midi_note: midi_note(F_S, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyG,
            label: 'G',
            midi_note: midi_note(G, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyY,
            label: 'Y',
            midi_note: midi_note(G_S, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyH,
            label: 'H',
            midi_note: midi_note(A, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyU,
            label: 'U',
            midi_note: midi_note(A_S, BASE_OCTAVE),
        },
        KeyBinding {
            key: Key::KeyJ,
            label: 'J',
            midi_note: midi_note(B, BASE_OCTAVE),
        },
    ]
}

pub fn render(active_keys: &HashSet<Key>) {
    let bindings = key_bindings();
    let mut buffer = String::new();
    const BLACK_KEY_CHAR: &str = "█";
    const BLACK_KEY_PRESSED: &str = "░";
    const WHITE_KEY_PRESSED: &str = "▒";
    const WHITE_KEY_EMPTY: &str = " ";
    buffer.push_str("\x1B[H");
    buffer.push_str("   W  E     T  Y  U \n");
    buffer.push_str("   ");
    for (i, label) in ['W', 'E', 'T', 'Y', 'U'].iter().enumerate() {
        if let Some(binding) = bindings.iter().find(|b| b.label == *label) {
            let name = note_name_no_octave(binding.midi_note);
            buffer.push_str(&format!("{:^3}", name));
            if i == 1 {
                buffer.push_str("   ");
            }
        }
    }
    buffer.push('\n');
    buffer.push_str("   ");
    for (i, label) in ['W', 'E', 'T', 'Y', 'U'].iter().enumerate() {
        let pressed = active_keys.contains(&key_for_label(*label));
        let ch = if pressed { BLACK_KEY_PRESSED } else { BLACK_KEY_CHAR };
        buffer.push_str(ch);
        buffer.push_str("  ");
        if i == 1 {
            buffer.push_str("   ");
        }
    }
    buffer.push_str("   \n");
    buffer.push_str("┌─┤");
    buffer.push_str(draw_black('W', active_keys));
    buffer.push_str("││");
    buffer.push_str(draw_black('E', active_keys));
    buffer.push_str("│││ │");
    buffer.push_str(draw_black('T', active_keys));
    buffer.push_str("││");
    buffer.push_str(draw_black('Y', active_keys));
    buffer.push_str("││");
    buffer.push_str(draw_black('U', active_keys));
    buffer.push_str("├┐\n");
    buffer.push_str("│ │");
    buffer.push_str(draw_black('W', active_keys));
    buffer.push_str("││");
    buffer.push_str(draw_black('E', active_keys));
    buffer.push_str("│││ │");
    buffer.push_str(draw_black('T', active_keys));
    buffer.push_str("││");
    buffer.push_str(draw_black('Y', active_keys));
    buffer.push_str("││");
    buffer.push_str(draw_black('U', active_keys));
    buffer.push_str("││\n");
    for label in ['A', 'S', 'D', 'F', 'G', 'H', 'J'] {
        let pressed = active_keys.contains(&key_for_label(label));
        buffer.push_str("│");
        buffer.push_str(if pressed { WHITE_KEY_PRESSED } else { WHITE_KEY_EMPTY });
        buffer.push_str("│");
    }
    buffer.push('\n');
    for _ in 0..7 {
        buffer.push_str("└─┘");
    }
    buffer.push('\n');
    for label in ['A', 'S', 'D', 'F', 'G', 'H', 'J'] {
        if let Some(binding) = bindings.iter().find(|b| b.label == label) {
            let name = note_name_no_octave(binding.midi_note);
            buffer.push_str(&format!("{:^3}", name));
        }
    }
    buffer.push('\n');
    print!("{}", buffer);
    let _ = stdout().flush();
}

fn draw_black(label: char, active_keys: &HashSet<Key>) -> &'static str {
    if active_keys.contains(&key_for_label(label)) {
        "░"
    } else {
        "█"
    }
}

fn key_for_label(label: char) -> Key {
    for binding in key_bindings() {
        if binding.label == label {
            return binding.key;
        }
    }
    panic!("Invalid label: {}", label);
}

fn note_name_no_octave(note_number: u8) -> &'static str {
    const NAMES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
    NAMES[(note_number % 12) as usize]
}

pub fn play_midi(midi_bytes: &[u8]) {
    const MIDI_NOTE_ON: u8 = 0x90;
    const MIDI_NOTE_OFF: u8 = 0x80;

    let (midi_out, port) = connect_to_first_midi_port();
    let mut conn = midi_out.connect(&port, "rust-midi").unwrap();
    let smf = Smf::parse(&midi_bytes).unwrap();
    let events = prepare_events(&smf);
    // const TARGET_CHANNEL: u8 = 0;
    // const PROGRAM: u8 = 0;
    // events = inject_program_change(events, TARGET_CHANNEL, PROGRAM);
    let mut last_time = 0.0;
    process_midi_events_with_timing(events, &smf, |event_time, event, ch| {
        let delay = event_time - last_time;
        if delay > 0.0 {
            thread::sleep(Duration::from_secs_f32(delay));
        }
        last_time = event_time;
        if let Some(channel) = ch {
            if let TrackEventKind::Midi { message, .. } = event {
                match message {
                    MidiMessage::NoteOn { key, vel } => {
                        let msg = if vel.as_int() > 0 {
                            vec![MIDI_NOTE_ON | channel, key.as_int(), vel.as_int()]
                        } else {
                            vec![MIDI_NOTE_OFF | channel, key.as_int(), 0]
                        };
                        let _ = conn.send(&msg);
                    },
                    MidiMessage::NoteOff { key, .. } => {
                        let msg = vec![MIDI_NOTE_OFF | channel, key.as_int(), 0];
                        let _ = conn.send(&msg);
                    },
                    _ => {},
                }
            }
        }
    });
}

const L0: &str = "";
const L1: &str = "  ├── ";
const L1_LAST: &str = "  └── ";
const L2: &str = "        ├── ";
const L2_LAST: &str = "        └── ";
const L3: &str = "        │   ├── ";
const L3_LAST: &str = "        │   └── ";
const L4: &str = "        │   │   ├── ";
const L4_LAST: &str = "        │   │   └── ";

pub fn print_full_structure(soundfont_file_path: &str, bank: i32, patch: i32) -> Result<(), Box<dyn Error>> {
    let file = File::open(soundfont_file_path)?;
    let mut reader = BufReader::new(file);
    let soundfont = SoundFont::new(&mut reader)?;
    let preset = soundfont
        .get_presets()
        .iter()
        .find(|p| p.get_bank_number() == bank && p.get_patch_number() == patch)
        .ok_or("No matching preset found in SoundFont.")?;
    let mut lines = Vec::new();
    print_preset_info(preset, &mut lines);
    let preset_regions = preset.get_regions();
    lines.push(format!("{L1}Preset Regions: {} bags", preset_regions.len()));
    for (i, preset_region) in preset_regions.iter().enumerate() {
        let is_last = i == preset_regions.len() - 1;
        let region_label = if is_last {
            format!("{L1_LAST}Preset Region index: {}", i)
        } else {
            format!("{L1}Preset Region index: {}", i)
        };
        lines.push(region_label);
        let instrument_index = preset_region.get_instrument_id();
        if let Some(instrument) = soundfont.get_instruments().get(instrument_index) {
            print_instrument_info(instrument, &soundfont, &mut lines);
        } else {
            lines.push(format!("{L2}(Missing instrument at index {})", instrument_index));
        }
    }
    print_aligned_right(&lines);
    Ok(())
}

fn print_preset_info(preset: &Preset, lines: &mut Vec<String>) {
    lines.push(format!("{L0}Preset: \"{}\"", preset.get_name()));
    lines.push(format!("{L1}Bank Number: {}", preset.get_bank_number()));
    lines.push(format!("{L1_LAST}Patch Number: {}", preset.get_patch_number()));
}

fn print_instrument_info(instrument: &Instrument, soundfont: &SoundFont, lines: &mut Vec<String>) {
    lines.push(format!("{L2}Instrument: \"{}\"", instrument.get_name()));
    let instrument_regions = instrument.get_regions();
    lines.push(format!("{L2}Instrument Regions: {} bags", instrument_regions.len()));
    for (i, region) in instrument_regions.iter().enumerate() {
        let is_last = i == instrument_regions.len() - 1;
        let label = if is_last {
            format!("{L2_LAST}Instrument Region index: {}", i)
        } else {
            format!("{L2}Instrument Region index: {}", i)
        };
        lines.push(label);
        print_instrument_region_key_velocity_info(region, lines);
        print_sample_info(region, soundfont, lines);
        print_region_pan_envelope(region, lines);
    }
}

fn print_instrument_region_key_velocity_info(region: &InstrumentRegion, lines: &mut Vec<String>) {
    let low = region.get_key_range_start() as u8;
    let high = region.get_key_range_end() as u8;
    let low_note = note_to_name(low);
    let high_note = note_to_name(high);
    lines.push(format!("{L3}Key Range: {}–{} ({}–{})", low, high, low_note, high_note));
    let vel_low = region.get_velocity_range_start();
    let vel_high = region.get_velocity_range_end();
    lines.push(format!(
        "{L3}Velocity Range: {}–{} (0=soft, 127=hard)",
        vel_low, vel_high
    ));
}

fn print_sample_info(region: &InstrumentRegion, soundfont: &SoundFont, lines: &mut Vec<String>) {
    let sample_id = region.get_sample_id();
    if let Some(sample) = soundfont.get_sample_headers().get(sample_id) {
        lines.push(format!("{L3}Sample: \"{}\"", sample.get_name()));
        lines.push(format!("{L4}Sample Rate: {} Hz", sample.get_sample_rate()));
        let loop_start = sample.get_start_loop();
        let loop_end = sample.get_end_loop();
        lines.push(format!("{L4}Loop: {} → {} ", loop_start, loop_end));
        let pitch = sample.get_original_pitch();
        let note = note_to_name(sample.get_original_pitch() as u8);
        lines.push(format!("{L4}Original Pitch: {} ({})", pitch, note));
        let pitch_correct = sample.get_pitch_correction();
        lines.push(format!("{L4}Pitch Correction: {} cents", pitch_correct));
        lines.push(format!("{L4_LAST}Sample Type: {} (1 = mono)", sample.get_sample_type()));
    }
}

fn print_region_pan_envelope(region: &InstrumentRegion, lines: &mut Vec<String>) {
    let pan = region.get_pan();
    let pan_desc = if pan == 0.0 {
        "Center".into()
    } else if pan < 0.0 {
        format!("Left {}% ({} pan)", (pan.abs() * 100.0) as i32, pan)
    } else {
        format!("Right {}% ({} pan)", (pan * 100.0) as i32, pan)
    };
    lines.push(format!("{L3}Pan Position: {} (Stereo balance)", pan_desc));
    let attack = region.get_attack_volume_envelope();
    lines.push(format!("{L3}Volume Envelope - Attack Time: {:.3} sec", attack));
    let decay = region.get_decay_volume_envelope();
    lines.push(format!("{L3}Volume Envelope - Decay Time: {:.3} sec", decay));
    let sustain = region.get_sustain_volume_envelope();
    lines.push(format!("{L3}Volume Envelope - Sustain Level: {:.1} dB", sustain));
    let envelope_vol = region.get_release_volume_envelope();
    lines.push(format!(
        "{L3_LAST}Volume Envelope - Release Time: {:.3} sec)",
        envelope_vol
    ));
}

fn print_aligned_right(lines: &[String]) {
    let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let keyboard_width = key_bindings().len() * 6;
    let gap = 1;
    let term_width = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80);
    let required = keyboard_width + gap + max_width;
    let pad = if required <= term_width {
        keyboard_width + gap
    } else {
        term_width.saturating_sub(max_width)
    };
    for line in lines {
        println!("{}{}", " ".repeat(pad), line);
    }
}

pub fn note_to_name(note_number: u8) -> String {
    const NAMES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
    let octave = (note_number / 12).saturating_sub(1);
    let name = NAMES[(note_number % 12) as usize];
    format!("{}{}", name, octave)
}
