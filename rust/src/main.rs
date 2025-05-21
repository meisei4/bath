mod audio_analysis;
mod collision_mask;
mod midi;

use crate::midi::playback::play_midi;
use midi::midi::parse_midi_events_into_note_on_off_event_buffer_TICKS;
use midi::sf_debug::print_full_structure;

const SOUND_FONT_FILE_PATH: &str = "/Users/ann/Documents/misc_game/Animal_Crossing_Wild_World.sf2";
const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";

//fluidsynth -a coreaudio -m coremidi /Users/ann/Documents/misc_game/Animal_Crossing_Wild_World.sf2

fn main() {
    //print!("\x1B[2J");
    //print_metadata(SOUND_FONT_FILE_PATH);
    if let Err(err) = print_full_structure(SOUND_FONT_FILE_PATH, 0, 0) {
        eprintln!("️SoundFont debug error: {}", err);
    }
    // // if let Err(err) = print_full_structure(SOUND_FONT_FILE_PATH, 1, 2) {
    // //     eprintln!("️SoundFont debug error: {}", err);
    // // }
    // let mut fluidsynth_process = launch_fluidsynth_with_font(SOUND_FONT_FILE_PATH);
    // let (midi_output, midi_port) = connect_to_first_midi_port();
    // let mut midi_connection = midi_output.connect(&midi_port, "rust-midi").unwrap();
    // let mut pressed_keys: HashSet<Key> = HashSet::new();
    // let _ = listen(move |event| {
    //     handle_key_event(event, &mut midi_connection, &mut pressed_keys);
    // });
    // let _ = fluidsynth_process.kill();

    let _ = parse_midi_events_into_note_on_off_event_buffer_TICKS(MIDI_FILE_PATH);
    play_midi(MIDI_FILE_PATH)
}
