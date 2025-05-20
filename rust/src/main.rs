use crate::midi::{connect_to_first_midi_port, handle_key_event, launch_fluidsynth_with_font};
use crate::sf_debug::{print_full_structure, print_metadata};
use rdev::listen;
use std::collections::HashSet;

mod keys;
mod midi;
mod sf_debug;
mod tsf_bindings;

const SOUND_FONT_FILE_PATH: &str = "/Users/ann/Downloads/Animal_Crossing_Wild_World.sf2";

fn main() {
    //print!("\x1B[2J");
    print_metadata(SOUND_FONT_FILE_PATH);
    if let Err(err) = print_full_structure(SOUND_FONT_FILE_PATH, 0, 0) {
        eprintln!("️SoundFont debug error: {}", err);
    }
    let mut fluidsynth_process = launch_fluidsynth_with_font(SOUND_FONT_FILE_PATH);
    let (midi_output, midi_port) = connect_to_first_midi_port();
    let mut midi_connection = midi_output.connect(&midi_port, "rust-midi").unwrap();
    let mut pressed_keys: HashSet<rdev::Key> = HashSet::new();
    let _ = listen(move |event| {
        handle_key_event(event, &mut midi_connection, &mut pressed_keys);
    });
    let _ = fluidsynth_process.kill();
}
