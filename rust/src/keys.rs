use rdev::Key;
use std::collections::HashSet;
use std::io::{stdout, Write};

pub struct KeyBinding {
    pub key: Key,
    pub label: char,
    pub midi_note: u8,
}

pub fn key_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding {
            key: Key::KeyA,
            label: 'A',
            midi_note: 60,
        },
        KeyBinding {
            key: Key::KeyS,
            label: 'S',
            midi_note: 62,
        },
        KeyBinding {
            key: Key::KeyD,
            label: 'D',
            midi_note: 64,
        },
        KeyBinding {
            key: Key::KeyF,
            label: 'F',
            midi_note: 65,
        },
        KeyBinding {
            key: Key::KeyG,
            label: 'G',
            midi_note: 67,
        },
        KeyBinding {
            key: Key::KeyH,
            label: 'H',
            midi_note: 69,
        },
        KeyBinding {
            key: Key::KeyJ,
            label: 'J',
            midi_note: 71,
        },
        KeyBinding {
            key: Key::KeyK,
            label: 'K',
            midi_note: 72,
        },
        KeyBinding {
            key: Key::KeyL,
            label: 'L',
            midi_note: 74,
        },
        KeyBinding {
            key: Key::SemiColon,
            label: ';',
            midi_note: 76,
        },
    ]
}

pub fn render(active_keys: &HashSet<Key>) {
    print!("\x1B[H\n\n\n\n");
    let notes_row: String = key_bindings()
        .iter()
        .map(|b| format!("{:^4}", note_to_name(b.midi_note)))
        .collect();
    println!("{}", notes_row);

    let keys_row: String = key_bindings()
        .iter()
        .map(|b| format!(" {}  ", b.label))
        .collect();
    println!("{}", keys_row);

    let top_border: String = key_bindings().iter().map(|_| "┌─┐ ").collect();
    println!("{}", top_border);

    let middle_row: String = key_bindings()
        .iter()
        .enumerate()
        .map(|(i, b)| {
            if active_keys.contains(&b.key) {
                if i % 2 == 0 {
                    "│░│ "
                } else {
                    "│▒│ "
                }
            } else {
                "│ │ "
            }
        })
        .collect();
    println!("{}", middle_row);
    println!("{}", middle_row);

    // 5) Bottom border
    let bottom_border: String = key_bindings().iter().map(|_| "└─┘ ").collect();
    println!("{}", bottom_border);

    let _ = stdout().flush();
}

pub fn note_to_name(note_number: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
    ];
    let octave = (note_number / 12).saturating_sub(1);
    let name = NAMES[(note_number % 12) as usize];
    format!("{}{}", name, octave)
}
