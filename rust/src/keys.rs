use rdev::Key;
use std::collections::HashSet;
use std::io::{stdout, Write};

pub struct KeyBinding {
    pub key: Key,
    pub label: char,
    pub midi_note: u8,
}

//this is just the middle of a piano i guess
// MIDI: C4 = 60
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
        KeyBinding { key: Key::KeyA, label: 'A', midi_note: midi_note(C, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyW, label: 'W', midi_note: midi_note(C_S, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyS, label: 'S', midi_note: midi_note(D, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyE, label: 'E', midi_note: midi_note(D_S, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyD, label: 'D', midi_note: midi_note(E, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyF, label: 'F', midi_note: midi_note(F, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyT, label: 'T', midi_note: midi_note(F_S, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyG, label: 'G', midi_note: midi_note(G, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyY, label: 'Y', midi_note: midi_note(G_S, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyH, label: 'H', midi_note: midi_note(A, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyU, label: 'U', midi_note: midi_note(A_S, BASE_OCTAVE) },
        KeyBinding { key: Key::KeyJ, label: 'J', midi_note: midi_note(B, BASE_OCTAVE) },
    ]
}
pub fn render(active_keys: &HashSet<rdev::Key>) {
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
                buffer.push_str("   "); // Extra space after second black key
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
            buffer.push_str("   "); // Extra space after second black key
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
    //buffer.push_str(" A  S  D  F  G  H  J \n");
    print!("{}", buffer);
    let _ = stdout().flush();
}

fn draw_black(label: char, active_keys: &HashSet<rdev::Key>) -> &'static str {
    if active_keys.contains(&key_for_label(label)) {
        "░"
    } else {
        "█"
    }
}

fn key_for_label(label: char) -> rdev::Key {
    for binding in key_bindings() {
        if binding.label == label {
            return binding.key;
        }
    }
    panic!("Invalid label: {}", label);
}

pub fn note_to_name(note_number: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
    ];
    let octave = (note_number / 12).saturating_sub(1);
    let name = NAMES[(note_number % 12) as usize];
    format!("{}{}", name, octave)
}

fn note_name_no_octave(note_number: u8) -> &'static str {
    const NAMES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
    NAMES[(note_number % 12) as usize]
}
