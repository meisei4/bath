use crate::keys::{key_bindings, render};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use rdev::{Event, EventType, Key};
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
